#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{RuntimeDebug, ArithmeticError};
use sp_std::{prelude::*, fmt::Debug};
use frame_system::{self as system, pallet_prelude::*,ensure_signed};
use orml_traits::{DataFeeder, DataProvider, MultiCurrency};
use frame_support::{pallet_prelude::*, transactional, dispatch, traits::Get, log,PalletId};
use frame_support::{
	traits::{InitializeMembers, EnsureOrigin, Contains,Currency,ReservableCurrency,ExistenceRequirement,OnUnbalanced,Imbalance},
	sp_runtime::{traits::{AccountIdConversion,Zero},FixedPointNumber},
};

use pallet_oracle::UpdateOraclesStorgage;
use pallet_amm::Pair;
// use frame_support::log;
// use serde::{Deserialize, Serialize};

use orml_utilities::with_transaction_result;
use sp_runtime::{DispatchError, DispatchResult, FixedU128};
// use support::{DEXManager, ExchangeRateProvider, Price, PriceProvider};
pub use primitives::{Price, CurrencyId, Balance, Moment, CORE_ASSET_ID};
// use frame_support::traits::Instance;

mod traits;
mod tests;
mod mock;
pub mod weights;

pub use weights::WeightInfo;

pub use module::*;
use traits::PriceProvider;
use sp_core::U256;

#[frame_support::pallet]
pub mod module {
	use super::*;
	use frame_support::storage::child::len;

	pub(crate) type BalanceOf<T, I = ()> =
	<<T as Config<I>>::BaseCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type NegativeImbalanceOf<T, I = ()> =
	<<T as Config<I>>::BaseCurrency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;


	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
	#[cfg_attr(test, derive(Default))]
	pub struct DepositBalanceInfo<Balance,BlockNum>
	{
		pub amount: Balance,
		pub expiration: BlockNum,  // If the value is 0, the expiration time is not set
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_amm::Config{
		// 	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
		type Source: DataProvider<CurrencyId, Price> + DataFeeder<CurrencyId, Price, Self::AccountId>;
		type FeedOrigin: EnsureOrigin<Self::Origin>;
		type MembershipInitialized: UpdateOraclesStorgage<Self::AccountId,CurrencyId>;
		type BaseCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type PledgedBalance:  Get<BalanceOf<Self,I>>;

		#[pallet::constant]
		type DicoTreasuryModuleId: Get<PalletId>;

		#[pallet::constant]
		type WithdrawExpirationPeriod: Get<Self::BlockNumber>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config<I>, I: 'static = ()> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locked_price)]
	pub type LockedPrice<T: Config<I>, I: 'static = ()> = StorageMap<_, Twox64Concat, CurrencyId, Price>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_balance)]
	pub type DepositBalance<T: Config<I>, I: 'static = ()> = StorageMap<_, Twox64Concat, T::AccountId, DepositBalanceInfo<BalanceOf<T,I>, T::BlockNumber>>;

	// #[pallet::genesis_config]
	// pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
	// 	pub members: Vec<T::AccountId>,
	// 	pub phantom: sp_std::marker::PhantomData<I>,
	// }
	//
	// #[cfg(feature = "std")]
	// impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
	// 	fn default() -> Self {
	// 		GenesisConfig {
	// 			members: Default::default(),
	// 			phantom: Default::default(),
	// 		}
	// 	}
	// }
	//
	// #[pallet::genesis_build]
	// impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
	// 	fn build(&self) {
	// 		// <Members<T, I>>::put(self.members.clone());
	// 		let mut members = self.members.clone();
	// 		members.sort();
	// 		T::MembershipInitialized::initialize_members(members.clone());
	// 		<Members<T, I>>::put(members);
	// 	}
	// }

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		LockPrice(CurrencyId, Price),
		UnlockPrice(CurrencyId),
		WhoLock(T::AccountId),
		/// Some accounts deposit successfully
		DepositAccounts(Vec<T::AccountId>,BalanceOf<T,I>),
		/// slash some accounts and transfer balance to treasury
		SlashAccounts(Vec<T::AccountId>),
		///
		GetPrice(CurrencyId,U256),
	}


	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Expiration time is empty
		ExpirationEmpty,

		ExpirationNotEmpty,
		/// Not expired
		NotExpired,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Council slash accounts
		#[pallet::weight((<T as Config<I>>::WeightInfo::del_feed_account(accounts.len() as u32), DispatchClass::Operational))]
		pub fn del_feed_account(origin: OriginFor<T>, accounts: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			T::FeedOrigin::ensure_origin(origin)?;
			let new_account = accounts.iter().filter(|&i| Self::slash(i.clone())).map(|i|i.clone()).
				collect::<Vec<_>>();
			T::MembershipInitialized::del_members(&new_account);
			Ok(().into())
		}

		#[pallet::weight((<T as Config<I>>::WeightInfo::insert_feed_account(accounts.len() as u32), DispatchClass::Operational))]
		pub fn insert_feed_account(origin: OriginFor<T>, accounts: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			T::FeedOrigin::ensure_origin(origin)?;
			// pledge
			let new_account = accounts.iter().filter(|&i| Self::deposit(i.clone(), T::PledgedBalance::get())).map(|i|i.clone()).
				collect::<Vec<_>>();
			// info::info!("-----------insert_feed_account account: {:?} {:?}-----------", &new_account,T::PledgedBalance::get());
			ensure!((new_account.len())!= 0,Error::<T,I>::NoneValue);
			T::MembershipInitialized::insert_members(&new_account);
			Self::deposit_event(Event::DepositAccounts(new_account, T::PledgedBalance::get()));
			Ok(().into())
		}

		#[pallet::weight((<T as Config<I>>::WeightInfo::unlock_price(), DispatchClass::Operational))]
		pub fn unlock_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			T::FeedOrigin::ensure_origin(origin)?;
			T::MembershipInitialized::unlock_price(currency_id);
			Self::deposit_event(Event::UnlockPrice(currency_id));
			Ok(().into())

		}

		#[pallet::weight((<T as Config<I>>::WeightInfo::exit_feed(), DispatchClass::Operational))]
		pub fn exit_feed(origin: OriginFor<T>) -> DispatchResultWithPostInfo{
			// only set expiration time
			let feeder = ensure_signed(origin.clone())?;
			Self::set_expiration(&feeder)?;
			T::MembershipInitialized::del_members(&[feeder]);
			Ok(().into())
		}

		#[pallet::weight((<T as Config<I>>::WeightInfo::withdraw(), DispatchClass::Operational))]
		pub fn withdraw(origin: OriginFor<T>) -> DispatchResultWithPostInfo{
			// withdraw owner ledge
			let feeder = ensure_signed(origin.clone())?;
			Self::do_withdraw(&feeder)?;
			Ok(().into())
		}

		#[pallet::weight((<T as Config<I>>::WeightInfo::get_price(), DispatchClass::Operational))]
		pub fn get_price(origin: OriginFor<T>,currency_id1: CurrencyId, currency_id2: CurrencyId) -> DispatchResultWithPostInfo {
			let price = <Self as PriceProvider<CurrencyId>>::get_price_from_swap(currency_id1,currency_id2)?;
			Self::deposit_event(Event::GetPrice(currency_id1,price));
			Ok(().into())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I>  {
	fn account_id() -> T::AccountId {
		T::DicoTreasuryModuleId::get().into_account()
	}

	fn deposit(who: T::AccountId, deposit_balance: BalanceOf<T,I>) -> bool{
		// let res = T::Currency::transfer(&who, &Self::account_id(),deposit_balance,ExistenceRequirement::KeepAlive);
		let res = T::BaseCurrency::reserve(&who,deposit_balance);
		match res {
			Ok(()) => {
				DepositBalance::<T, I>::insert(&who,DepositBalanceInfo{amount: deposit_balance, expiration: T::BlockNumber::from(0u32)});
				true
			},
			_ => {
				log::error!("transfer have error: {:?}",res);
				false
			},
		}
	}

	fn set_expiration(who: &T::AccountId) -> DispatchResult{
		// The expiration time is set by the user
		let balance_info: Option<DepositBalanceInfo<_, _>> = <DepositBalance<T, I>>::get(who);
		match balance_info{
			Some(balance_info) => {
				if !balance_info.expiration.is_zero(){ // is not zero: in the exiting
					return Err(Error::<T,I>::ExpirationNotEmpty)?;
				}
			},
			_ => return Err(Error::<T,I>::NoneValue)?,  // not exist
		};
		let expiration = system::Pallet::<T>::block_number() + T::WithdrawExpirationPeriod::get();
		<DepositBalance::<T, I>>::mutate(who,|d|{
			if let Some(d) = d{
				d.expiration = expiration};
		});

		Ok(())
	}

	fn slash(who: T::AccountId) -> bool{
		// Transfer of funds to the treasury
		let balance_info: Option<DepositBalanceInfo<_, _>> = <DepositBalance<T, I>>::get(&who);
		let info = match balance_info{
			Some(balance_info) => {
				balance_info},
			_ => return false,
		};
		T::BaseCurrency::unreserve(&who,info.amount);
		let res = T::BaseCurrency::transfer(&who, &Self::account_id(),info.amount,ExistenceRequirement::AllowDeath);
		match res{
			Ok(_) => {
				<DepositBalance<T, I>>::remove(&who);
				true},
			Err(_) => false
		}
	}

	fn do_withdraw(who: &T::AccountId) -> DispatchResult{
		let now = system::Pallet::<T>::block_number();
		let balance_info: Option<DepositBalanceInfo<_, _>> = <DepositBalance<T, I>>::get(who);
		let info = match balance_info{
			Some(balance_info) => {
				if balance_info.expiration.is_zero(){ // is zero
					return Err(Error::<T,I>::ExpirationEmpty)?;
				}
				if now < balance_info.expiration{
					return Err(Error::<T,I>::NotExpired)?;
				}
				balance_info},
			_ => return Err(Error::<T,I>::NoneValue)?,
		};
		T::BaseCurrency::unreserve(who,info.amount);
		// delete DepositBalanceInfo
		<DepositBalance<T, I>>::remove(who);
		return Ok(());
	}
}

impl<T: Config<I>, I: 'static> PriceProvider<CurrencyId> for Pallet<T, I> {
	type Price = Price;
	fn get_price_from_swap(currency_id1: CurrencyId, currency_id2: CurrencyId) -> sp_std::result::Result<U256, ArithmeticError>{
		// currency_id1: the queried currency
		// currency_id2: stable coin's currency id,such as usdt
		let liquidity = pallet_amm::Pallet::<T>::get_liquidity(Pair::new(currency_id1,currency_id2)).ok_or(ArithmeticError::DivisionByZero)?;
		if currency_id1 < currency_id2 {
			pallet_amm::math::get_amount_out(U256::from(1),U256::from(liquidity.0),U256::from(liquidity.1))
		}else{
			pallet_amm::math::get_amount_out(U256::from(1),U256::from(liquidity.1),U256::from(liquidity.0))
		}
	}
}



