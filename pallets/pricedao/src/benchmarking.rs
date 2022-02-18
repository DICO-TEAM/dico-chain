#![cfg(feature = "runtime-benchmarks")]

use super::*;

pub use crate::Pallet as PriceDao;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{Account, Origin, Pallet as System, RawOrigin};
use orml_traits::MultiCurrencyExtended;
use pallet_oracle::Pallet as DicoOracle;
use primitives::{AssetId, Balance};
use sp_std::prelude::*;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	// <T  as StoredMap<T::AccountId, T::AccountData>>::insert(&caller, AccountData { free,
	// ..Default::default() }).unwrap(); <T as ::Config>::AccountStore::insert(&caller, AccountData
	// {free:DEFAULT_BALANCE, ..Default::default() }).unwrap(); T::Currency::update_balance(1, &caller,
	// 1_000_000_000_000_000).unwrap();
	let pledge_deposit = T::PledgedBalance::get();
	let _ = T::BaseCurrency::make_free_balance_be(&caller, pledge_deposit.saturating_add(2000_000_000u32.into()));
	caller
}
fn on_initialize_insert_members<T: Config>() -> Vec<T::AccountId> {
	let BOB = funded_account::<T>("caller", 0);
	let ALICE = funded_account::<T>("caller", 1);
	let DAVA = funded_account::<T>("caller", 2);
	let JEY = funded_account::<T>("caller", 3);
	let origin = T::FeedOrigin::successful_origin();
	// DicoOracle::<T,_>::feed_values(accounts[1], vec![(3, 1400)])?;
	let accs = vec![BOB.clone(), ALICE.clone(), DAVA.clone(), JEY.clone()];
	PriceDao::<T>::insert_feed_account(origin, accs.clone());
	return accs;
}

benchmarks! {
	insert_feed_account {
		let BOB = funded_account::<T>("caller", 1);
		let origin = T::FeedOrigin::successful_origin();
		// let call = Call::<T>::insert_feed_account(vec![BOB]);
	}: _<T::Origin>(origin, vec![BOB.clone()])
	verify {
		assert_eq!(T::BaseCurrency::free_balance(&BOB), 2000000000u32.into());
		assert_eq!(DepositBalance::<T>::get(&BOB).unwrap(),
			crate::DepositBalanceInfo::<_,_>{amount: T::PledgedBalance::get(),expiration: 0u32.into()});
	}

	del_feed_account {
		let BOB = funded_account::<T>("caller", 1);
		let origin = T::FeedOrigin::successful_origin();
		PriceDao::<T>::insert_feed_account(origin,vec![BOB.clone()]);
		let origin = T::FeedOrigin::successful_origin();

	}: _<T::Origin>(origin, vec![BOB.clone()])
	verify {
		// assert_eq!(T::BaseCurrency::total_balance(&BOB), 2000000000u32.into());
		assert_eq!(DepositBalance::<T>::get(&BOB), None);
		assert_eq!(T::BaseCurrency::total_balance(&PriceDao::<T>::account_id()),T::PledgedBalance::get());
	}

	unlock_price {
		let accounts = on_initialize_insert_members::<T>();
		let origin = T::FeedOrigin::successful_origin();
	}: _<T::Origin>(origin, 1)
	verify {

	}

	exit_feed {
		let BOB = funded_account::<T>("caller", 1);
		let origin = T::FeedOrigin::successful_origin();
		PriceDao::<T>::insert_feed_account(origin,vec![BOB.clone()]);
	}: _(RawOrigin::Signed(BOB.clone()))
	verify {
		let num = System::<T>::block_number() + T::WithdrawExpirationPeriod::get();
		assert_eq!(T::BaseCurrency::free_balance(&BOB), 2000000000u32.into());
		assert_eq!(crate::DepositBalance::<T>::get(&BOB),
				   Some(crate::DepositBalanceInfo{amount: T::PledgedBalance::get(),expiration: num}));
	}

	withdraw {
		let BOB = funded_account::<T>("caller", 1);
		let origin = T::FeedOrigin::successful_origin();
		PriceDao::<T>::insert_feed_account(origin,vec![BOB.clone()]);
		PriceDao::<T>::exit_feed(RawOrigin::Signed(BOB.clone()).into());

		let num = System::<T>::block_number() + T::WithdrawExpirationPeriod::get();
		System::<T>::set_block_number(num +T::BlockNumber::from(1u32));
	}: _(RawOrigin::Signed(BOB.clone()))
	verify {
		assert_eq!(T::BaseCurrency::free_balance(&BOB), T::PledgedBalance::get()+2000000000u32.into());

	}


}

impl_benchmark_test_suite!(PriceDao, crate::mock::new_test_ext(), crate::mock::Test,);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_insert_feed_account() {
		new_test_ext().execute_with(|| {
			println!("***********************");
			assert_ok!(PriceDao::<Test>::test_benchmark_insert_feed_account());
		});
	}

	#[test]
	fn test_del_feed_account() {
		new_test_ext().execute_with(|| {
			assert_ok!(PriceDao::<Test>::test_benchmark_del_feed_account());
		});
	}

	#[test]
	fn test_unlock_price() {
		new_test_ext().execute_with(|| {
			assert_ok!(PriceDao::<Test>::test_benchmark_unlock_price());
		});
	}

	#[test]
	fn test_exit_feed() {
		new_test_ext().execute_with(|| {
			assert_ok!(PriceDao::<Test>::test_benchmark_exit_feed());
		});
	}

	#[test]
	fn test_withdraw() {
		new_test_ext().execute_with(|| {
			assert_ok!(PriceDao::<Test>::test_benchmark_withdraw());
		});
	}
}
