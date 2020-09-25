

use sp_std::{prelude::*, result::Result, collections::{btree_set::BTreeSet, btree_map::BTreeMap}};
use frame_support::{debug, ensure, decl_module, decl_storage, decl_error, decl_event, weights::{Weight},
					StorageValue, StorageMap, StorageDoubleMap, Blake2_256, traits::{WithdrawReason, Get, ExistenceRequirement::AllowDeath, IcoAsset, Currency, ReservableCurrency, LockIdentifier, LockableCurrency}};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_runtime::{DispatchResult, Percent, ModuleId, RuntimeDebug, traits::{AccountIdConversion, CheckedAdd, One}};
use codec::{Encode, Decode};
use node_primitives::{USDT, Balance};
use pallet_balances::{self as balances};
use pallet_generic_asset::{self as generic_asset, NextAssetId, AssetOptions, PermissionsV1, PermissionLatest,
						   TotalIssuance, FreeBalance, ReservedBalance, Permissions, };
use pallet_identity::{self as identity};
use crate::raw::{Additional, Address, AddressEnum, TokenAmount, RaiseAmount, Symbol, IcoInfo};
use sp_std::convert::{TryInto,TryFrom, Into};

type BalanceOf<T> = <<T as identity::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub const DICO_ID: LockIdentifier = *b"dico    ";

pub trait Trait: system::Trait + balances::Trait + generic_asset::Trait + identity::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type ModuleId: Get<ModuleId>;

	// 募集资金需要抵押的金额
	type RaiseDeposit: Get<BalanceOf<Self>>;

	// 募集资金需要的最小占币
	type MinProportion: Get<Percent>;

	// 募集资金最大的周期
	type MaxDurtion: Get<Self::BlockNumber>;
}


decl_storage! {
	trait Store for Module<T: Trait> as IcoModule {

		/// 所有正在参加ico或是已经ico成功的项目 (project_name, symbol) => (asset_id, end_time, IcoInfo)
		pub Projects get(fn all_project): double_map hasher(blake2_128_concat) Vec<u8>, hasher(blake2_128_concat) Vec<u8> => Option<(Additional<T::AssetId, T::BlockNumber, BTreeSet<T::AccountId>>, IcoInfo<T::GenericBalance, T::BlockNumber, Address<T::AccountId>>)>;

		/// 资产id对应的币种(todo 多资产模块初始化币种应该对这个也进行初始化)
		pub SymbolOf get(fn symbol_of): map hasher(blake2_128_concat) T::AssetId => Option<(Vec<u8>, Vec<u8>)>;

		/// 正在进行ico的项目（资产id代替）
		pub Raising get(fn raising): BTreeSet<T::AssetId>;

		/// 项目筹集资金的具体金额
		pub SpecificRaiseAmount get(fn specific_raise_amount): map hasher(blake2_128_concat) T::AssetId => Option<RaiseAmount<TokenAmount<AddressEnum<T::AccountId>>, BTreeMap<T::AccountId, TokenAmount<AddressEnum<T::AccountId>>>>>;

		/// 正在进行dao的项目
		pub Dao get(fn dao): BTreeSet<T::AssetId>;
	}
	}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {
		/// 数据溢出
		Overflow,
		/// 可抵押金额不够
		BongTooLow,
		/// 字符串参数为空
		StringEmpty,
		/// 代币数目错误
		TokenAmountErr,
		/// usdt数量错误
		UsdtAmountErr,
		/// 达到最大周期要求
		ToMaxDurtion,
		/// 地址为空
		AddressEmpty,
		/// 不是正在ico的资产
		NotRaising,
		/// 获取数据错误
		GetErr,
		/// 过期
		Expire,
		/// usdt超出限额
		AmountTooLarge,
		/// usdt没有达到最低
		AmountTooShort,
		/// 金额参数不能是0
		AmountZero,
		/// 在被排除在外的国家
		InExcludeCountry,
		/// 不支持的代币
		UnknownSymbol,
		/// 没有相关的代币地址
		AddressNotExists,

	}
	}


decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		/// 国库的id
		const ModuleId: ModuleId = T::ModuleId::get();
		/// 募集资金最长时间
		const MaxDurtion: T::BlockNumber = T::MaxDurtion::get();
		/// 达到多少占比才能募集资金成功
		const MinProportion: Percent = T::MinProportion::get();
		/// 募集资金需要抵押的金额
		const RaiseDeposit: BalanceOf<T> = T::RaiseDeposit::get();

		type Error = Error<T>;
		fn deposit_event() = default;


		/// 项目方要求募集资金
		#[weight = 120_000_000]
		fn ask_for_raise(origin, info: IcoInfo<T::GenericBalance, T::BlockNumber, Address<T::AccountId>>){
			let who = ensure_signed(origin)?;
			let mut info = info.clone();
			// 字符串相关参数不能为空
			if info.project_name.clone().len() == 0
				|| info.symbol.clone().len() == 0
				|| info.logo_url.clone().len() == 0
				|| info.official_website.clone().len() == 0
				{
						return Err(Error::<T>::StringEmpty)?;
					}

			// 总发行量与流通量、用于募资量大于0 并且流通量小于发行量， 用于募资的量
			if info.total_issuance.clone() == T::GenericBalance::from(0u32)
				|| info.total_circulation.clone() == T::GenericBalance::from(0u32)
				|| info.total_circulation.clone() > info.total_issuance.clone()
				|| info.total_token_in_use == T::GenericBalance::from(0)
				|| info.total_token_in_use.clone() > info.total_issuance.clone()
				{
					return Err(Error::<T>::TokenAmountErr)?;
				}

			// 如果最大的usdt量与最小都有 那么最大的不能小与最小的
			// 如果最小的usdt存在 那么最小的usdt不应该大于实际需要募集的
			if info.user_min_usdt.is_some(){
				if info.user_max_usdt.clone().is_some(){
					if info.user_max_usdt.clone().unwrap() <= info.user_min_usdt.clone().unwrap(){
						return Err(Error::<T>::UsdtAmountErr)?;
				}
				}

				if info.user_min_usdt.clone().unwrap() > info.raise_usdt_total.clone(){
					return Err(Error::<T>::UsdtAmountErr)?;
				}

			}

			// 募集的usdt数量不能是0
			ensure!(info.raise_usdt_total.clone() != (0 as USDT), Error::<T>::UsdtAmountErr);

			// 接收筹款的地址不能是空
			ensure!((info.public_keys.usdt.clone().is_some() && info.public_keys.usdt.clone().unwrap().len() != 0)
				|| (info.public_keys.dico.clone().is_some()),
				Error::<T>::AddressEmpty
			);

			let project_name = info.project_name.clone();
			let symbol = info.symbol.clone();

			// 判断这个项目是否正在参加ico或是已经成功ico
			if !<Projects<T>>::contains_key(project_name.clone(), symbol.clone()){

				// 进行抵押
				T::Currency::reserve(&who, T::RaiseDeposit::get()).map_err(|_| Error::<T>::BongTooLow)?;

				// 获取下一个资产id
				let id = Self::get_next_asset_id();

				// 募集资金的周期不能大于系统最长
				ensure!(info.raise_duration.clone() <= T::MaxDurtion::get(), Error::<T>::ToMaxDurtion);

				let asset_options = AssetOptions {
					/// 募集的资金作为当前的总发行量
					initial_issuance: info.total_token_in_use.clone(),
					/// 不给铸币 更新 销毁权限
					permissions: PermissionLatest::<T::AccountId>::default(),
				};

				// 创建资产到default账户
				Self::create_asset(id.clone(), None, asset_options)?;

				let end_time = Self::now() + info.raise_duration.clone();

				<Raising<T>>::mutate(|z| z.insert(id.clone()));

				<SymbolOf<T>>::insert(id.clone(), (project_name.clone(), symbol.clone()));

				<Projects<T>>::insert(project_name.clone(), symbol.clone(), (Additional{asset_id: id.clone(), end_time: end_time.clone(), already_raise_usdt: 0 as USDT, people: BTreeSet::<T::AccountId>::default()}, info.clone()));

				// 设置下一个资产id
				Self::set_next_asset_id();

				Self::deposit_event(RawEvent::AskForRaised(who, project_name, symbol));
			}

		}


		/// 项目方取消筹集资金(等于项目出问题， 取消本次募集资金)
		#[weight = 120_000_000]
		fn cancel_raise(origin, asset_id: T::AssetId){
			let who = ensure_signed(origin)?;
			// 这个项目正在筹集资金
			let raising = <Raising<T>>::get();
			if raising.contains(&asset_id) {
				Self::fail_to_do(asset_id);
			}

			else {
				return Err(Error::<T>::NotRaising)?;
			}

		}


		/// 项目方中途决定关闭募集资金（钱已经筹集足够，不再继募集）
		#[weight = 120_000_000]
		fn close_raise(origin, asset_id: T::AssetId) {
			let who = ensure_signed(origin)?;
			// 项目正在募集资金
			let mut raising = <Raising<T>>::get();
			if raising.contains(&asset_id) {
				let (project_name, symbol) = <SymbolOf<T>>::get(asset_id.clone()).unwrap();
				let info = <Projects<T>>::get(project_name, symbol).unwrap();
				let raise_usdt_total = info.1.raise_usdt_total.clone();
				let already_raise_usdt = info.0.already_raise_usdt.clone();
				if T::MinProportion::get() * raise_usdt_total <= already_raise_usdt {
					let mut dao = <Dao<T>>::get();
					dao.insert(asset_id.clone());
					<Dao<T>>::put(dao);

					raising.remove(&asset_id);
					<Raising<T>>::put(raising);

				}

			}
			else {
				return Err(Error::<T>::NotRaising)?;
			}
		}


		/// 用户参与ico
		#[weight = 120_000_000]
		fn user_join_into_ico(origin, asset_id: T::AssetId, user_symbol: Symbol, usdt_amount: USDT) {

			let who = ensure_signed(origin)?;

			let amount = usdt_amount;

			// amount参数不能是0
			ensure!(amount > 0 as USDT, Error::<T>::AmountZero);

			// 这个资产是否正在ico
			ensure!(<Raising<T>>::get().contains(&asset_id), Error::<T>::NotRaising);

			let (project_name, symbol) = <SymbolOf<T>>::get(asset_id.clone()).ok_or(Error::<T>::GetErr)?;

			// 获取ico具体信息
			let mut info = <Projects<T>>::get(project_name.clone(), symbol.clone()).ok_or(Error::<T>::GetErr)?;

			// todo 判断是否已经过期 过期要进行相应的处理（归还币 销毁资产 删除Raising数据）
			if Self::now() > info.0.end_time.clone() {
				Self::fail_to_do(asset_id.clone());
				return Err(Error::<T>::Expire)?;
			}
			//
			// 累加金额不能大于最大募集资金
			let amount1 = amount.checked_add(info.0.already_raise_usdt.clone()).ok_or(Error::<T>::Overflow)?;
			ensure!(info.1.raise_usdt_total.clone() >= amount1, Error::<T>::NotRaising);

			// 金额不能小于最小（也不能是0）  不能大于最大
			if info.1.user_max_usdt.clone().is_some(){
				ensure!(info.1.user_max_usdt.clone().unwrap() <= amount.clone(), Error::<T>::AmountTooLarge);
			}
			if info.1.user_min_usdt.clone().is_some(){
				ensure!(info.1.user_min_usdt.clone().unwrap() >= amount.clone(), Error::<T>::AmountTooShort);
			}

			// todo 被排除在外的国家不能参与（结合identity模块)
			ensure!(!Self::is_exclude_countries(who.clone(), info.1.exclude_countries.clone()), Error::<T>::InExcludeCountry);


			// todo usdt转换成代币数量
			let token = Self::usdt_convert_to_balances(user_symbol.clone(), amount.clone());

			// todo 代币琐仓

			// 对项目方进行琐仓
			Self::set_lock_for_manager(user_symbol.clone(), token, info.0.clone(), info.1.clone())?;

			// todo 存储个人筹集资金记录（币种 金额 地址）

			// 存储项目具体金额
			let already_raise_usdt = info.0.already_raise_usdt.clone();
			let now_amount = already_raise_usdt + usdt_amount;
			info.0.already_raise_usdt = now_amount;
			<Projects<T>>::insert(project_name, symbol, info.clone());

			// 判断筹集资金是否结束（金额到顶） 结束直接处理(删除Raising中的数据, 在Dao中添加该资产id)
			if amount1 == info.1.raise_usdt_total {
				let mut raising = <Raising<T>>::get();
				raising.remove(&asset_id);
				<Raising<T>>::put(raising);
				let mut dao = <Dao<T>>::get();
				dao.insert(asset_id.clone());
				<Dao<T>>::put(dao);
			}
		}


		/// 募集资金过程中(募集结果还没有定下来)， 用户退出
		#[weight = 120_000_000]
		fn user_quit_ico(origin, project_id: u32) {

		}


		fn on_finalize(n: T::BlockNumber){
			let raising = <Raising<T>>::get();
			let mut raising_iter = raising.iter();
			let len = raising.clone().len();
			for i in 0..len {
				let asset_id = raising_iter.next().unwrap();
				let symbol_info = <SymbolOf<T>>::get(asset_id.clone());
				if symbol_info.is_some() {
					let (project_name, symbol) = symbol_info.unwrap();
					let project_opt = <Projects<T>>::get(project_name, symbol);
					if project_opt.is_some() {
						let now = Self::now();
						let end_time = project_opt.clone().unwrap().0.end_time;
						// 如果结束时间到
						if now > end_time {

							// 如果募集的资金达到要求
							if T::MinProportion::get() * project_opt.clone().unwrap().1.raise_usdt_total <= project_opt.clone().unwrap().0.already_raise_usdt {
								let mut dao = <Dao<T>>::get();
								dao.insert(asset_id.clone());
								<Dao<T>>::put(dao);

								let mut raising_1 = <Raising<T>>::get();
								raising_1.remove(&asset_id);
								<Raising<T>>::put(raising_1);
							}

						}
						else {
							Self::fail_to_do(*asset_id);
						}
					}
					else {
						continue;
					}
				}
				else{
					continue;
				}
			}
		}

}
}


decl_event!(
	pub enum Event<T> where
	 <T as system::Trait>::AccountId {
		AskForRaised(AccountId, Vec<u8>, Vec<u8>),
	}
);


impl <T: Trait> Module<T> {

	/// 获取国库id
	pub fn get_treasury_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}


	/// 获取下一个资产id
	pub fn get_next_asset_id() -> T::AssetId {
		<generic_asset::Module<T>>::next_asset_id()
	}

	/// 设置下一个资产id
	pub fn set_next_asset_id() -> DispatchResult{
		let mut asset_id = Self::get_next_asset_id();
		asset_id = asset_id.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
		<NextAssetId<T>>::put(asset_id);
		Ok(())
	}


	/// 创建一个资产(先处理下一个asset_id后才会成功执行)
	pub fn create_asset(
		asset_id: T::AssetId,
		from_account: Option<T::AccountId>,
		options: AssetOptions<T::GenericBalance, T::AccountId>,
		) -> DispatchResult{

		<generic_asset::Module<T>>::create_asset(Some(asset_id), from_account, options)?;
		Ok(())
	}


	/// 获取当前区块高度
	fn now() -> T::BlockNumber{
		<system::Module<T>>::block_number()
	}


	// todo 把其他币种的金额转换成usdt
	fn usdt_convert_to_balances(user_symbol: Symbol, usdt_amount: USDT) -> Balance{
		10000 as Balance
	}


	// todo 这个人是否在被排除在kyc外的国家
	fn is_exclude_countries(who: T::AccountId, countrise: Vec<Vec<u8>>) -> bool{
		false

	}


	// todo 销毁多资产模块的某个币种
	fn remove_asset(asset_id: T::AssetId) {
		<TotalIssuance<T>>::remove(asset_id);
		<Permissions<T>>::remove(asset_id);

	}


	/// 募集资金没有成功 处理
	fn fail_to_do(asset_id: T::AssetId) {
		let (project_name, symbol) = <SymbolOf<T>>::get(asset_id.clone()).unwrap();
		let info = <Projects<T>>::get(project_name.clone(), symbol.clone()).unwrap();

		let people = info.0.people;
		// 获取本项目目的具体筹集
		let specific_raise_amount = <SpecificRaiseAmount<T>>::get(asset_id.clone()).unwrap();

		// 对募集资金的项目方进行解锁
		Self::release_project_manager_lock(specific_raise_amount.clone());
		// 归还每个参与ico的人员的相应币种(删除SpecificRaiseAmount信息)
		Self::give_back_player_token(specific_raise_amount.clone(), people.clone());

		// 删除具体筹集资金的信息
		<SpecificRaiseAmount<T>>::remove(asset_id.clone());

		// 销毁该项目方的资产
		Self::remove_asset(asset_id.clone());

		// 删除SymbolOf
		<SymbolOf<T>>::remove(asset_id.clone());

		// 删除Projects
		<Projects<T>>::remove(project_name.clone(), symbol.clone());

		// 从正在ico的队列种删除
		let mut raising = <Raising<T>>::get();
		raising.remove(&asset_id);
		<Raising<T>>::put(raising);
	}


	/// 释放项目方的琐仓
	fn release_project_manager_lock(raise_amount: RaiseAmount<TokenAmount<AddressEnum<T::AccountId>>, BTreeMap<T::AccountId, TokenAmount<AddressEnum<T::AccountId>>>>) {
		// 获取项目方募集到的具体资金情况
		let project_manager_get = raise_amount.project_manager_get;

		// 对项目方募集到的具体币种进行解锁处理
		if let Some(dico) = project_manager_get.dico {
			match dico.0 {
				AddressEnum::<T::AccountId>::Dico(x) => {
					T::Currency::remove_lock(DICO_ID, &x);
				},
				_ => {},

		}

		}

		// todo 其他币种也要进行释放 目前等待xcmp上线再进行处理

	}


	/// 归还每个参与ico的人员的相应币种
	fn give_back_player_token(raise_amount: RaiseAmount<TokenAmount<AddressEnum<T::AccountId>>, BTreeMap<T::AccountId, TokenAmount<AddressEnum<T::AccountId>>>>, people: BTreeSet<T::AccountId>) {
		let people_cp = people.clone();
		let mut people_iter = people.iter();
		// 获取项目方的信息
		let project_manager_get = raise_amount.project_manager_get.clone();
		// 获取参与ico的人员的信息
		let others_send = raise_amount.others_send.clone();

		for i in 0..people_cp.len(){
			if let Some(man) = people_iter.next() {
				// 查找个人的具体筹集资金的情况
				if let Some(other_info) = others_send.get(&man) {

					// 处理dico
					{
						let manager_dico = project_manager_get.dico.clone();
						let other_dico = &other_info.dico;

						// 这两地址都存在才有意义
						if manager_dico.is_none() || other_dico.is_none() {
							continue;
						}

						// 获取项目方的地址
						let manager_address = match manager_dico.unwrap().0.clone() {
							AddressEnum::<T::AccountId>::Dico(x) => Some(x),
							_ => None,
						};
						// 获取这个人的地址
						let other_address = match &other_dico.as_ref().unwrap().0 {
							AddressEnum::<T::AccountId>::Dico(x) => Some(x),
							_ => None,
						};

						let mut dico_amount = <BalanceOf<T>>::from(0u32);
						if <BalanceOf<T> as TryFrom::<Balance>>::try_from(other_dico.as_ref().unwrap().1).ok().is_some() {
							dico_amount = <BalanceOf<T> as TryFrom::<Balance>>::try_from(other_dico.as_ref().unwrap().1).ok().unwrap();
						}

						if manager_address.is_some() && other_address.is_some() {
							T::Currency::transfer(&manager_address.unwrap(), &other_address.unwrap(), dico_amount, AllowDeath);
						}
					}

					// todo 其他币种等待xcmp上线后处理
					{
					}

			}
			}

		}
	}


	/// 给项目方进行琐仓
	fn set_lock_for_manager(symbol: Symbol, amount: Balance, additional: Additional<T::AssetId, T::BlockNumber, BTreeSet<T::AccountId>>, ico_info: IcoInfo<T::GenericBalance, T::BlockNumber, Address<T::AccountId>>) -> DispatchResult {
		// 获取项目方的所有地址
		let address = ico_info.public_keys.clone();

		match symbol {
			Symbol::Dico => {

				// 如果项目方的dico地址不存在 就结束
				if address.dico.is_some() {
					return Err(Error::<T>::AddressNotExists)?;
				}

				let dico_address = address.dico.unwrap();

				let asset_id = additional.asset_id;

				// 获取具体信息
				let raise_opt = <SpecificRaiseAmount<T>>::get(asset_id.clone());
				// 如果私募的信息已经存在
				if raise_opt.is_some() {
					let specific_raise_amount = raise_opt.unwrap();
					let project_manager_get = specific_raise_amount.project_manager_get.clone();
					let manager_dico = project_manager_get.dico.clone();

					// 如果dico的信息存在
					if manager_dico.clone().is_some() {
						let manager_address = match &manager_dico.as_ref().unwrap().0 {
							AddressEnum::<T::AccountId>::Dico(x) => Some(x),
							_ => return Err(Error::<T>::AddressNotExists)?,
						};

						let dico = manager_dico.clone().unwrap();

						// 如果dico的地址存在 增加dico数量 以及增加usdt数量
						let real_amount = dico.1 + amount;
						let real_amount = <BalanceOf<T> as TryFrom::<Balance>>::try_from(real_amount).ok().unwrap();
						let reasons = WithdrawReason::Transfer | WithdrawReason::Reserve;
						T::Currency::set_lock(DICO_ID, &manager_address.unwrap(), real_amount, reasons);

					}
					// 如果dico的信息不存在

				}

				// 如果是第一次添加私募信息
				else {
					let real_amount = amount;
					let real_amount = <BalanceOf<T> as TryFrom::<Balance>>::try_from(real_amount).ok().unwrap();
					let reasons = WithdrawReason::Transfer | WithdrawReason::Reserve;
					T::Currency::set_lock(DICO_ID, &dico_address, real_amount, reasons);

				}

				},

			_ => return Err(Error::<T>::UnknownSymbol)?,
		}
		Ok(())
	}




}



impl<T: Trait> IcoAsset<T::AssetId> for Module<T>{
	fn set_asset_symbol(id: T::AssetId, project_name: Vec<u8>, symbol: Vec<u8>){
		<SymbolOf<T>>::insert(id, (project_name, symbol));
	}
}
