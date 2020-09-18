
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
use crate::raw::{Additional, Address, AddressEnum, TokenAmount, RaiseAmount, Symbol, IcoInfo};
use crate::ico;


type BalanceOf<T> = <<T as identity::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;


pub trait Trait: system::Trait + ico::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_storage! {
	trait Store for Module<T: Trait> as DaoModule {

	}
	}


decl_error! {
	/// Error for the elections module.
	pub enum Error for Module<T: Trait> {

	}
	}


decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;
		fn deposit_event() = default;

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

