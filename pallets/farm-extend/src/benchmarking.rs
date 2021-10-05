#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_std::prelude::*;

use crate::Pallet as FarmExtend;

use primitives::AssetId;

const SEED: u32 = 1;
const DICO: AssetId = 1000;
const DOT: AssetId = 2000;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::update_balance(DICO, &caller, 1_000_000_000_000_000).unwrap();
	T::Currency::update_balance(DOT, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	create_pool {
		let caller = funded_account::<T>("caller", 0);

	}: _(RawOrigin::Signed(caller.clone()), DOT, T::BlockNumber::from(100u32), T::BlockNumber::from(1100u32), 1_000_000_000u128, DICO)
	verify {
		assert_eq!(T::Currency::free_balance(DOT, &caller), 999000000000000u128);
	}

	deposit_asset {
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);

		FarmExtend::<T>::create_pool(
			RawOrigin::Signed(maker.clone()).into(),
			DOT,
			T::BlockNumber::from(100u32),
			T::BlockNumber::from(1100u32),
			1_000_000_000u128,
			DICO
		)?;

	}: _(RawOrigin::Signed(caller.clone()), T::PoolExtendId::from(0u32), 1000_000_000_000u128)
	verify {
		assert_eq!(T::Currency::free_balance(DOT, &maker), 999000000000000u128);
		assert_eq!(T::Currency::free_balance(DICO, &caller), 999000000000000u128);
	}

	withdraw_asset {
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);
		FarmExtend::<T>::create_pool(
			RawOrigin::Signed(maker.clone()).into(),
			DOT,
			T::BlockNumber::from(100u32),
			T::BlockNumber::from(1100u32),
			1_000_000_000u128,
			DICO
		)?;

		FarmExtend::<T>::deposit_asset(
			RawOrigin::Signed(caller.clone()).into(),
			T::PoolExtendId::from(0u32),
			1000_000_000_000u128
		)?;

	}: _(RawOrigin::Signed(caller.clone()), T::PoolExtendId::from(0u32), 1000_000_000_000u128)
	verify {
		assert_eq!(T::Currency::free_balance(DOT, &maker), 999000000000000u128);
		assert_eq!(T::Currency::free_balance(DICO, &caller), 1_000_000_000_000_000u128);
	}
}
