#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::StoredMap;
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;
use pallet_balances::AccountData;
use sp_std::prelude::*;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	<T as StoredMap<T::AccountId, T::AccountData>>::insert(
		who,
		AccountData {
			free,
			..Default::default()
		},
	)
	.unwrap();
	// T::Currency::update_balance(1, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	insert_feed_account {
		let origin = T::FeedOrigin::successful_origin();
		let BOB = funded_account::<T>("caller", 1);
		let call = Call::<T>::insert_feed_account(vec![BOB]);
	}: {call.dispatch_bypass_filter(origin)?}
	verify {
		assert_eq!(T::BaseCurrency::free_balance(&BOB), 0);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			println!("bench");
			assert_ok!(test_benchmark_insert_feed_account::<Test>());
		});
	}
}
