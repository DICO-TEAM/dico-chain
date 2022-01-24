#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks_instance, impl_benchmark_test_suite, benchmarks, whitelisted_caller};
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use sp_std::vec;
use crate::Pallet as Currencies;

const SEED: u32 = 0;

fn get_alice<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &caller, 10000u32.into());
	caller
}

fn get_bob<T: Config>() -> T::AccountId {
	let caller: T::AccountId = account("bob", 1, SEED);
	T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &caller, 10000u32.into());
	caller
}

fn look_up<T: Config>(who: T::AccountId) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(who)
}


fn create<T: Config>() -> (T::AccountId, u32) {
	let Alice = get_alice::<T>();
	let currency_id = 50;
	assert!(Currencies::<T>::create_asset(RawOrigin::Signed(Alice.clone()).into(), currency_id, BalanceOf::<T>::from(100000u32), None).is_ok());
	(Alice, currency_id)
}

fn get_asset<T: Config>() -> (T::AccountId, u32) {
	let (owner, id) = create::<T>();
	assert!(Currencies::<T>::set_metadata(RawOrigin::Signed(owner.clone()).into(), id, DicoAssetMetadata {
		name: vec![1; 4],
		symbol: vec![1; 4],
		decimals: 12
	}).is_ok());
	(owner, id)
}


benchmarks! {
	create_asset {
		let Alice = get_alice::<T>();
		let currency_id = 50;
	}:_(RawOrigin::Signed(Alice.clone()), currency_id, BalanceOf::<T>::from(100000u32), None)
	verify {
		assert!(!T::MultiCurrency::total_issuance(currency_id).is_zero())
	}

	set_metadata {
		let (alice, id) = create::<T>();

	}:_(RawOrigin::Signed(alice.clone()), id, DicoAssetMetadata {
		name: vec![1;4],
		symbol: vec![1;4],
		decimals: 12
	})

	burn {
		let (alice, id) = get_asset::<T>();

	}:_(RawOrigin::Signed(alice.clone()), id, BalanceOf::<T>::from(10000u32))

	transfer {
		let (alice, id) = get_asset::<T>();
		let bob = get_bob::<T>();

	}:_(RawOrigin::Signed(alice.clone()), look_up::<T>(bob.clone()), id, BalanceOf::<T>::from(10000u32))

	transfer_native_currency {
		let alice = get_alice::<T>();
		let bob = get_bob::<T>();
	}:_(RawOrigin::Signed(alice.clone()), look_up::<T>(bob.clone()),  BalanceOf::<T>::from(10u32))

	update_balance {
		let (alice, id) = get_asset::<T>();
	}:_(RawOrigin::Root,  look_up::<T>(alice.clone()), id, AmountOf::<T>::from(10u32))
}

impl_benchmark_test_suite!(Currencies, crate::mock::new_test_ext(), crate::mock::Test);

