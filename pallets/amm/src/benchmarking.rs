#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::prelude::*;

use crate::Pallet as AMM;

use dico_currencies::{DicoAssetMetadata, Pallet as CurrenciesPallet};
use dico_primitives::currency::DOLLARS;
use dico_primitives::{AssetId, Balance};

pub const KSM: AssetId = 1000;
pub const DOT: AssetId = 2000;

const SEED: u32 = 1;

fn preset_asset<T: Config>() {
	let caster: T::AccountId = account("caster", 10, SEED);

	assert_ok!(T::Currency::deposit(0, &caster, 10000 * DOLLARS));

	assert_ok!(T::CurrenciesHandler::do_create(
		caster.clone(),
		DOT,
		Some(DicoAssetMetadata {
			name: "Polkadot".into(),
			symbol: "DOT".into(),
			decimals: 10
		}),
		0,
		false
	));

	assert_ok!(T::CurrenciesHandler::do_create(
		caster.clone(),
		KSM,
		Some(DicoAssetMetadata {
			name: "Kusama".into(),
			symbol: "KSM".into(),
			decimals: 10
		}),
		0,
		false
	));
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::update_balance(KSM, &caller, 1_000_000_000_000_000).unwrap();
	T::Currency::update_balance(DOT, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	add_liquidity {
		preset_asset::<T>();
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = DOT;
		let asset_b: AssetId = KSM;
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
		preset_asset::<T>();
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = DOT;
		let asset_b: AssetId = KSM;
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
		preset_asset::<T>();
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = DOT;
		let asset_b: AssetId = KSM;
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
		preset_asset::<T>();
		let maker = funded_account::<T>("maker", 0);
		let caller = funded_account::<T>("caller", 0);

		let asset_a: AssetId = DOT;
		let asset_b: AssetId = KSM;
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
