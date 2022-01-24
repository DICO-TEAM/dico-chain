#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller,impl_benchmark_test_suite};
use frame_system::{Pallet as System,RawOrigin,Account,Origin};
use sp_std::prelude::*;
use primitives::{AssetId, Balance,CurrencyId};
pub use crate::Pallet as DicoOracle;

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    <DicoOracle<T> as UpdateOraclesStorgage<T::AccountId, T::OracleKey>>::insert_members(&[caller.clone()]);
    caller
}


benchmarks! {
	feed_values {
		let BOB = funded_account::<T>("caller", 1);
		// let call = Call::<T>::insert_feed_account(vec![BOB]);
	}: _(RawOrigin::Signed(BOB.clone()), vec![(1u32.into(), 1300u32.into())])
	verify {
        assert_eq!(HasDispatched::<T, _>::get().contains(&BOB),true);
	}
}

impl_benchmark_test_suite!(DicoOracle, crate::mock::new_test_ext(), crate::mock::Test,);




#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_feed_valaue() {
        new_test_ext().execute_with(|| {
            println!("***********************");
            assert_ok!(DicoOracle::<Test>::test_benchmark_feed_values());
        });
    }
}