
use sp_std::{prelude::*, result::Result, collections::{btree_set::BTreeSet, btree_map::BTreeMap}};
use frame_support::{debug, ensure, decl_module, decl_storage, decl_error, decl_event, weights::{Weight},
					StorageValue, StorageMap, StorageDoubleMap, Blake2_256, traits::{Get, IcoAsset, Currency, ReservableCurrency}};
use frame_system as system;
use system::{ensure_signed, ensure_root};
use sp_runtime::{DispatchResult, Percent, ModuleId, RuntimeDebug, traits::{AccountIdConversion, CheckedAdd, One}};
use codec::{Encode, Decode};
use node_primitives::{USDT, Balance};
use pallet_balances::{self as balances};
use pallet_generic_asset::{self as generic_asset, NextAssetId, AssetOptions};
use pallet_identity::{self as identity};
use crate::raw::*;
use crate::ico;

type BalanceOf<T> = <<T as identity::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;


pub trait Trait: system::Trait + generic_asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// 退出ico提案的过期时间
	type QuitExpire: Get<Self::BlockNumber>;
	/// 释放琐仓的提案的过期时间
	type ReleaseExpire: Get<Self::BlockNumber>;
	///  通过这个可以直接操作ico模块
	type IcoHandler: IcoHandler<Self::AssetId,
		Additional<Self::AssetId, Self::BlockNumber, BTreeSet<Self::AccountId>>,
		IcoInfo<Self::GenericBalance, Self::BlockNumber, Address<Self::AccountId>>,
		RaiseAmount<TokenAmount<AddressEnum<Self::AccountId>>, BTreeMap<Self::AccountId, TokenAmount<AddressEnum<Self::AccountId>>>>>;

}


decl_storage! {
	trait Store for Module<T: Trait> as DaoModule {

		/// 正在进行dao的项目
		pub Dao get(fn dao): BTreeSet<T::AssetId>;
	}
	}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {

	}
	}


decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
// 		/// 退出ico的提案的过期时间
// 		const QuitExpire: T::BlockNumber = T::QuitExpire::get();
// 		/// 释放琐仓提案的过期时间
// 		const ReleaseExpire: T::BlockNumber = T::ReleaseExpire::get();

		type Error = Error<T>;
		fn deposit_event() = default;

		/// 提出彻底退出募集资金的议案
		#[weight = 120_000_000]
		fn request_quit(origin, asset_id: T::AssetId) {
			// 签名
			//

		}

		/// 给退出ico的提案进行投票
		#[weight = 120_000_000]
		fn vote_for_quit_proposal(origin, asset_id: T::AssetId, vote: DicoVote){

		}

		/// 项目方请求释放琐仓(释放哪个币种)
		#[weight = 120_000_000]
		fn request_release(origin, asset_id: T::AssetId, symbol: Symbol, amount: Balance) {

		}

		/// 对释放琐仓的提案进行投票
		#[weight = 120_000_000]
		fn vote_for_release_proposal(origin, asset_id: T::AssetId) {

		}



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

