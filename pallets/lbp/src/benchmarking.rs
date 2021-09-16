#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use sp_std::prelude::*;
use frame_system::{RawOrigin};

use crate::Pallet as Lbp;

use dico_primitives::{AssetId};

const SEED: u32 = 1;

const DOT: AssetId = 1;
const USDC: AssetId = 2;
pub const WEIGHT_ONE: u128 = 10_000_000_000u128;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::update_balance(DOT, &caller, 10_000_000_000_000_000_000_000_000_000).unwrap();
	T::Currency::update_balance(USDC, &caller, 10_000_000_000_000_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	create_lbp {
		let caller = funded_account::<T>("caller", 0);

		let afs_asset: AssetId = USDC;
		let fundraising_asset: AssetId = DOT;
		let afs_balance = 10_000_000_000_000u128;
		let fundraising_balance = 100_000_000_000_000u128;
		let afs_start_weight = 10u128 * WEIGHT_ONE;
		let afs_end_weight = 90u128 * WEIGHT_ONE;
		let fundraising_start_weight = 10u128 * WEIGHT_ONE;
		let fundraising_end_weight = 90u128 * WEIGHT_ONE;
		let start_block = T::BlockNumber::from(100u32);
		let end_block = T::BlockNumber::from(1000u32);
		let steps = BlockNumber::from(100u32);
	}: _(RawOrigin::Signed(caller.clone()), afs_asset, fundraising_asset, afs_balance,
		fundraising_balance, afs_start_weight, afs_end_weight, fundraising_start_weight,
		fundraising_end_weight, start_block, end_block, steps)
	verify {
		assert_eq!(T::Currency::free_balance(fundraising_asset, &caller), 10_000_000_000_000_000_000_000_000_000 - 100_000_000_000_000);
		assert_eq!(T::Currency::free_balance(afs_asset, &caller), 10_000_000_000_000_000_000_000_000_000 - 10_000_000_000_000);
	}

	exit_lbp {
		let caller = funded_account::<T>("caller", 0);

		Lbp::<T>::create_lbp(
			RawOrigin::Signed(caller.clone()).into(),
			USDC,
			DOT,
			100_000_000_000_000u128,
			10_000_000_000_000u128,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			T::BlockNumber::from(100u32),
			T::BlockNumber::from(1000u32),
			BlockNumber::from(100u32),
		)?;
		assert_eq!(T::Currency::free_balance(USDC, &caller), 10_000_000_000_000_000_000_000_000_000 - 100_000_000_000_000);
		assert_eq!(T::Currency::free_balance(DOT, &caller), 10_000_000_000_000_000_000_000_000_000 - 10_000_000_000_000);

	}: _(RawOrigin::Signed(caller.clone()), T::LbpId::zero())
	verify {
		assert_eq!(T::Currency::free_balance(USDC, &caller), 10_000_000_000_000_000_000_000_000_000);
		assert_eq!(T::Currency::free_balance(DOT, &caller), 10_000_000_000_000_000_000_000_000_000);
	}

	swap_exact_amount_supply {
		let saler = funded_account::<T>("caller", 0);
		let buyer = funded_account::<T>("caller", 1);

		Lbp::<T>::create_lbp(
			RawOrigin::Signed(saler.clone()).into(),
			USDC,
			DOT,
			1333333000000000000000000u128,
			7500000000000000000000000u128,
			4 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			4 * WEIGHT_ONE,
			T::BlockNumber::from(0u32),
			T::BlockNumber::from(1000u32),
			BlockNumber::from(100u32),
		)?;
		assert_eq!(T::Currency::free_balance(USDC, &saler), 10_000_000_000_000_000_000_000_000_000 - 1333333000000000000000000);
		assert_eq!(T::Currency::free_balance(DOT, &saler), 10_000_000_000_000_000_000_000_000_000 - 7500000000000000000000000);

	}: _(RawOrigin::Signed(buyer.clone()), USDC, 86034000000000000000000u128, DOT, 0)
	verify {
		assert_eq!(T::Currency::free_balance(USDC, &buyer), 10_000_000_000_000_000_000_000_000_000 - 86034000000000000000000);
		assert_eq!(T::Currency::free_balance(DOT, &buyer), 10_000_000_000_000_000_000_000_000_000 + 51927050621361330000000);
	}

	swap_exact_amount_target {
		let saler = funded_account::<T>("caller", 0);
		let buyer = funded_account::<T>("caller", 1);

		Lbp::<T>::create_lbp(
			RawOrigin::Signed(saler.clone()).into(),
			USDC,
			DOT,
			1333333000000000000000000u128,
			7500000000000000000000000u128,
			4 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			4 * WEIGHT_ONE,
			T::BlockNumber::from(0u32),
			T::BlockNumber::from(1000u32),
			BlockNumber::from(100u32),
		)?;
		assert_eq!(T::Currency::free_balance(USDC, &saler), 10_000_000_000_000_000_000_000_000_000 - 1333333000000000000000000);
		assert_eq!(T::Currency::free_balance(DOT, &saler), 10_000_000_000_000_000_000_000_000_000 - 7500000000000000000000000);

	}: _(RawOrigin::Signed(buyer.clone()), USDC, 986034000000000000000000u128, DOT, 51927050621361330000000u128)
	verify {
		assert_eq!(T::Currency::free_balance(USDC, &buyer), 10_000_000_000_000_000_000_000_000_000 - 86033999974477294587667);
		assert_eq!(T::Currency::free_balance(DOT, &buyer), 10_000_000_000_000_000_000_000_000_000 + 51927050621361330000000);
	}
}

