#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Currencies;
use frame_benchmarking::{account, benchmarks, benchmarks_instance, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::runtime_print;
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use sp_std::vec;

const SEED: u32 = 0;

fn get_alice<T: Config>() -> T::AccountId {
	runtime_print!("get alice");
	let caller: T::AccountId = whitelisted_caller();
	T::NativeCurrency::deposit(&caller, (10000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	runtime_print!(
		"amount: {:?}",
		<T as module::Config>::MultiCurrency::free_balance(T::GetNativeCurrencyId::get(), &caller)
	);
	caller
}

fn get_bob<T: Config>() -> T::AccountId {
	let caller: T::AccountId = account("bob", 1, SEED);
	T::NativeCurrency::deposit(&caller, (10000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	caller
}

fn look_up<T: Config>(who: T::AccountId) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(who)
}

fn create<T: Config>() -> (T::AccountId, u32) {
	let Alice = get_alice::<T>();
	let currency_id = 50;
	assert!(Currencies::<T>::create_asset(
		RawOrigin::Signed(Alice.clone()).into(),
		currency_id,
		(100000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
		None
	)
	.is_ok());
	(Alice, currency_id)
}

fn get_asset<T: Config>() -> (T::AccountId, u32) {
	let (owner, id) = create::<T>();
	assert!(Currencies::<T>::set_metadata(
		RawOrigin::Signed(owner.clone()).into(),
		id,
		DicoAssetMetadata {
			name: vec![1; 4],
			symbol: vec![1; 4],
			decimals: 12
		}
	)
	.is_ok());
	(owner, id)
}

benchmarks! {
	create_asset {
		let Alice = get_alice::<T>();
		let currency_id = 50;
	}:_(RawOrigin::Signed(Alice.clone()), currency_id, (100000 * DOLLARS).saturated_into::<BalanceOf<T>>(), None)
	verify {
		assert!(!<T as module::Config>::MultiCurrency::total_issuance(currency_id).is_zero())
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

	}:_(RawOrigin::Signed(alice.clone()), id, (100 * DOLLARS).saturated_into::<BalanceOf<T>>())

	transfer {
		let (alice, id) = get_asset::<T>();
		let bob = get_bob::<T>();

	}:_(RawOrigin::Signed(alice.clone()), look_up::<T>(bob.clone()), id, (100 * DOLLARS).saturated_into::<BalanceOf<T>>())

	transfer_native_currency {
		let alice = get_alice::<T>();
		let bob = get_bob::<T>();
	}:_(RawOrigin::Signed(alice.clone()), look_up::<T>(bob.clone()),  (10 * DOLLARS).saturated_into::<BalanceOf<T>>())

	update_balance {
		let (alice, id) = get_asset::<T>();
	}:_(RawOrigin::Root,  look_up::<T>(alice.clone()), id, (10 * DOLLARS).saturated_into::<AmountOf<T>>())
}

#[cfg(test)]
mod test1 {
	use super::*;
	use crate::mock::{new_test_ext, Runtime};
	use frame_support::assert_ok;
	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			get_alice::<Runtime>();
			// assert_ok!(Currencies::<Runtime>::test_benchmark_set_metadata());
			assert_ok!(Currencies::<Runtime>::test_benchmark_update_balance());
		});
	}
}
// impl_benchmark_test_suite!(Currencies, crate::mock::new_test_ext(), crate::mock::Test);
