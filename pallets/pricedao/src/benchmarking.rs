#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller,impl_benchmark_test_suite};
use frame_system::{RawOrigin,Account,Origin};
use sp_std::prelude::*;
use frame_support::{traits::{StoredMap}};
use pallet_balances::AccountData;
use primitives::{AssetId, Balance};
use orml_traits::MultiCurrencyExtended;
pub use crate::Pallet as PriceDao;
pub const DEFAULT_BALANCE: Balance = 2000_000_000_000_000_000;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    // <T  as StoredMap<T::AccountId, T::AccountData>>::insert(&caller, AccountData { free, ..Default::default() }).unwrap();
    // <T as ::Config>::AccountStore::insert(&caller, AccountData {free:DEFAULT_BALANCE, ..Default::default() }).unwrap();
    // T::Currency::update_balance(1, &caller, 1_000_000_000_000_000).unwrap();
    caller
}



benchmarks! {
	insert_feed_account {
        let caller: T::AccountId = whitelisted_caller();
		let BOB = funded_account::<T>("caller", 1);
		// let call = Call::<T>::insert_feed_account(vec![BOB]);
	}: _(RawOrigin::Root,vec![BOB.clone()])
	verify {
        println!("+++++++++++{:?}++++++++++++++++++",T::BaseCurrency::free_balance(&BOB));
		assert_eq!(T::BaseCurrency::free_balance(&BOB), 0u32.into());
	}
}

impl_benchmark_test_suite!(PriceDao, crate::mock::new_test_ext(), crate::mock::Test,);




#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            println!("***********************");
            assert_ok!(PriceDao::<Test>::test_benchmark_insert_feed_account());
        });
    }
}
