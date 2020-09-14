
use sp_std::{prelude::*, result::Result, collections::btree_set::BTreeSet};
use frame_support::{debug, ensure, decl_module, decl_storage, decl_error, decl_event, weights::{Weight},
					StorageValue, StorageMap, StorageDoubleMap, Blake2_256, traits::{Get, IcoAsset, Currency, ReservableCurrency}};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_runtime::{DispatchResult, Percent, ModuleId, RuntimeDebug, traits::{AccountIdConversion, CheckedAdd, One}};
use codec::{Encode, Decode};
use node_primitives::{USDT};
use pallet_balances as balances;
use pallet_generic_asset::{self as generic_asset, NextAssetId, AssetOptions};


type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// 募集资金的信息
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct IcoInfo<Balance>{
	/// 项目名字
	project_name: Vec<u8>,
	/// 代币名称
	symbol: Vec<u8>,
	/// logo地址
	logo_url: Vec<u8>,
	/// 项目是否做了认证
	is_identity: bool,
	/// 总发行量
	total_issuance: Balance,
	/// 总流通量
	total_circulation: Balance,
	/// 官网地址
	official_website: Vec<u8>,
	/// 用户参与的最小金额
	user_min_usdt: Option<USDT>,
	/// 用户参与的最大金额
	user_max_usdt: Option<USDT>,
	/// 这次募集资金拿出来的代币数量
	total_token_in_use: Balance,
	/// 募集的usdt数量
	raise_usdt_total: USDT,
	/// 排除在外的国家
	exclude_countries: Vec<Vec<u8>>,
	/// 琐仓比例
	vesting_proportion: Percent,
	/// 单次解锁比例
	unlock_proportion: Percent,
}

pub trait Trait: system::Trait + balances::Trait + generic_asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type ModuleId: Get<ModuleId>;

	// 募集资金需要抵押的金额
	type RaiseDeposit: Get<BalanceOf<Self>>;

	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	// 募集资金需要的最小占币
	type MinProportion: Get<Percent>;

	// 募集资金最大的周期
	type MaxDurtion: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {

		/// 所有正在参加ico或是已经ico成功的项目 (project_name, symbol) => (asset_id, end_time, IcoInfo)
		pub Projects get(fn all_project): double_map hasher(blake2_128_concat) Vec<u8>, hasher(blake2_128_concat) Vec<u8> => Option<(T::AssetId, T::BlockNumber, IcoInfo<T::Balance>)>;

		/// 资产id对应的币种(todo 多资产模块初始化币种应该对这个也进行初始化)
		pub SymbolOf get(fn symbol_of): map hasher(blake2_128_concat) T::AssetId => Option<(Vec<u8>, Vec<u8>)>;

		/// 正在进行ico的项目（资产id代替）
		pub Raising get(fn raising): BTreeSet<T::AssetId>;


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
		fn ask_for_raise(origin, info: IcoInfo<T::Balance>){
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
			if info.total_issuance.clone() == T::Balance::from(0u32)
				|| info.total_circulation.clone() == T::Balance::from(0u32)
				|| info.total_circulation.clone() > info.total_issuance.clone()
				|| info.total_token_in_use == T::Balance::from(0)
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

			let project_name = info.project_name.clone();
			let symbol = info.symbol.clone();

			// 判断这个项目是否正在参加ico或是已经成功ico
			if !<Projects<T>>::contains_key(project_name.clone(), symbol.clone()){

				// 进行抵押
				T::Currency::reserve(&who, T::RaiseDeposit::get()).map_err(|_| Error::<T>::BongTooLow)?;

				// 获取下一个资产id
				let id = Self::get_next_asset_id();

				let end_time = Self::now() + T::MaxDurtion::get();

				<Raising<T>>::mutate(|z| z.insert(id.clone()));

				<SymbolOf<T>>::insert(id.clone(), (project_name.clone(), symbol.clone()));

				<Projects<T>>::insert(project_name.clone(), symbol.clone(), (id.clone(), end_time.clone(), info.clone()));

				// 设置下一个资产id
				Self::set_next_asset_id();

				Self::deposit_event(RawEvent::AskForRaised(who, project_name, symbol));
			}

		}


		/// 项目方取消筹集资金(等于项目出问题， 取消本次募集资金)
		#[weight = 120_000_000]
		fn cancel_raise(origin, project_id: u32){
		}


		/// 项目方中途决定关闭募集资金（钱已经筹集足够，不再继募集）
		#[weight = 120_000_000]
		fn close_raise(origin, project_id: u32) {
		}


		/// 用户参与ico
		#[weight = 120_000_000]
		fn user_join_into_ico(origin, project_id: u32, amount: USDT) {

		}


		/// 募集资金过程中(募集结果还没有定下来)， 用户退出
		#[weight = 120_000_000]
		fn user_quit_ico(origin, project_id: u32) {

		}



// 		/// 检查ico是否过期， 过期看筹集资金是否达到最低要求
// 		/// 检查ico是否已经达到上限， 达到上限即给每个账号打币
// 		/// 问题： 怎么归还已经筹集到的币种？？？
		fn on_finalize(n: T::BlockNumber){
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

}

impl<T: Trait> IcoAsset<T::AssetId> for Module<T>{
	fn set_asset_symbol(id: T::AssetId, project_name: Vec<u8>, symbol: Vec<u8>){
		<SymbolOf<T>>::insert(id, (project_name, symbol));
	}
}
