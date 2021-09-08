#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_std::prelude::*;

use crate::Pallet as AMM;

use primitives::{AssetId, Balance};

const SEED: u32 = 1;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::update_balance(1, &caller, 1_000_000_000_000_000).unwrap();
	T::Currency::update_balance(2, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	add_liquidity {
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = 1;
		let asset_b: AssetId = 2;
		let amount_a_desired: Balance = 500_000_000_000_000;
		let amount_b_desired: Balance = 500_000_000_000_000;
		let amount_a_min: Balance = 400_000_000_000_000;
		let amount_b_min: Balance = 400_000_000_000_000;

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min)
	verify {
		assert_eq!(T::Currency::free_balance(asset_a, &caller), 500_000_000_000_000);
		assert_eq!(T::Currency::free_balance(asset_b, &caller), 500_000_000_000_000);
	}

	remove_liquidity {
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = 1;
		let asset_b: AssetId = 2;
		let liquidity_amount: Balance = 100_000_000_000_000 - 1000;

		AMM::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			asset_a, asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			80_000_000_000_000,
			80_000_000_000_000
		)?;
		assert_eq!(T::Currency::free_balance(asset_a, &caller), 900_000_000_000_000);
		assert_eq!(T::Currency::free_balance(asset_b, &caller), 900_000_000_000_000);

	}: _(RawOrigin::Signed(caller.clone()), asset_a, asset_b, liquidity_amount, 0, 0)
	verify {
		assert_eq!(T::Currency::free_balance(asset_a, &caller), 1_000_000_000_000_000 - 1000);
		assert_eq!(T::Currency::free_balance(asset_b, &caller), 1_000_000_000_000_000 - 1000);
	}

	swap_exact_assets_for_assets {
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = 1;
		let asset_b: AssetId = 2;
		let amount_in: Balance = 10_000_000_000_000;

		AMM::<T>::add_liquidity(
			RawOrigin::Signed(maker.clone()).into(),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			80_000_000_000_000,
			80_000_000_000_000
		)?;
		assert_eq!(T::Currency::free_balance(asset_a, &maker), 900_000_000_000_000);
		assert_eq!(T::Currency::free_balance(asset_b, &maker), 900_000_000_000_000);

	}: _(RawOrigin::Signed(caller.clone()), amount_in, 0, vec![asset_a, asset_b])
	verify {
		assert_eq!(T::Currency::free_balance(asset_a, &caller), 1_000_000_000_000_000 - 10_000_000_000_000);
		assert_eq!(T::Currency::free_balance(asset_b, &caller), 1_000_000_000_000_000 + 9_066_108_938_801);
	}

	swap_assets_for_exact_assets {
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = 1;
		let asset_b: AssetId = 2;
		let amount_out: Balance = 10_000_000_000_000;

		AMM::<T>::add_liquidity(RawOrigin::Signed(
			maker.clone()).into(),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			80_000_000_000_000,
			80_000_000_000_000
		)?;
		assert_eq!(T::Currency::free_balance(asset_a, &maker), 900_000_000_000_000);
		assert_eq!(T::Currency::free_balance(asset_b, &maker), 900_000_000_000_000);

	}: _(RawOrigin::Signed(caller.clone()), amount_out, 100_000_000_000_000, vec![asset_a, asset_b])
	verify {
		assert_eq!(T::Currency::free_balance(asset_a, &caller), 1_000_000_000_000_000 - 11_144_544_745_348);
		assert_eq!(T::Currency::free_balance(asset_b, &caller), 1_000_000_000_000_000 + 10_000_000_000_000);
	}
}

