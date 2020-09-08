
use sp_std::{prelude::*, result::Result};
use frame_support::{debug, ensure, decl_module, decl_storage, decl_error, decl_event, weights::{Weight},
					StorageValue, StorageMap, StorageDoubleMap, Blake2_256, traits::{Get}};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_runtime::{DispatchResult, Percent, ModuleId, RuntimeDebug, traits::{AccountIdConversion, CheckedAdd, One}};
use codec::{Encode, Decode};
use node_primitives::{USDT};
use pallet_balances as balances;
use pallet_generic_asset::{self as generic_asset, NextAssetId, AssetOptions};


/// 募集资金的信息
#[cfg_attr(feature = "std", derive())]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct IcoInfo<Balance, BlockNumber>{
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
	/// 募集资金周期
	raise_duration: BlockNumber,
	/// 用户参与的最小金额
	user_min_usdt: Option<USDT>,
	/// 用户参与的最大金额
	user_max_usdt: Option<USDT>,
	/// 这次募集资金拿出来的代币数量
	total_token_in_use: Balance,
	/// 募集的usdt数量
	raise_usdt_total: USDT,
	/// 募集资金成功要求的最小比例
	min_proportion: Percent,
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
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {



	}
	}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {
		/// 数据溢出
		Overflow,

	}
	}


decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		const ModuleId: ModuleId = T::ModuleId::get();
		type Error = Error<T>;
		fn deposit_event() = default;

		/// 项目方要求募集资金
		#[weight = 120_000_000]
		fn ask_for_raise(origin, info: IcoInfo<T::Balance, T::BlockNumber>){


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
		Ico(AccountId),
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

}
