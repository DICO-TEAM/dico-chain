#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::UnfilteredDispatchable;
use frame_system::{Pallet as System, RawOrigin};
use sp_core::U256;
use sp_std::prelude::*;

use crate::Pallet as Farm;

use dico_primitives::{AssetId, Balance};

const SEED: u32 = 1;
const LIQUIDITY_ID: AssetId = 3;

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::update_balance(1, &caller, 1_000_000_000_000_000).unwrap();
	T::Currency::update_balance(2, &caller, 1_000_000_000_000_000).unwrap();
	T::Currency::update_balance(LIQUIDITY_ID, &caller, 1_000_000_000_000_000).unwrap();
	caller
}

benchmarks! {
	set_halving_period {
	}: _(RawOrigin::Root, T::BlockNumber::from(1000u32))
	verify {
		assert_eq!(HalvingPeriod::<T>::get(), T::BlockNumber::from(1000u32));
	}

	set_dico_per_block {
	}: _(RawOrigin::Root, Balance::from(10000u32))
	verify {
		assert_eq!(DicoPerBlock::<T>::get(), Balance::from(10000u32));
	}

	set_start_block {
	}: _(RawOrigin::Root, T::BlockNumber::from(10000u32))
	verify {
		assert_eq!(StartBlock::<T>::get(), T::BlockNumber::from(10000u32));
	}

	create_pool {
		let _who = funded_account::<T>("caller", 0);
		let alloc_point = 1000u128;
	}: _(RawOrigin::Root, LIQUIDITY_ID, alloc_point)
	verify {
		assert_eq!(TotalAllocPoint::<T>::get(), alloc_point);
		let pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1);
		assert_eq!(Pools::<T>::get(T::PoolId::zero()).unwrap(), pool_info);
	}

	update_pool_alloc_point {
		let initial_alloc_point = 1000u128;
		let update_alloc_point = 10000u128;
		let pool_id = T::PoolId::zero();
		let _who = funded_account::<T>("caller", 0);

		Farm::<T>::create_pool(RawOrigin::Root.into(), LIQUIDITY_ID, initial_alloc_point)?;
	}: _(RawOrigin::Root, pool_id, update_alloc_point)
	verify {
		assert_eq!(TotalAllocPoint::<T>::get(), update_alloc_point);
		let pool_info = PoolInfo::new(LIQUIDITY_ID, update_alloc_point, 1);
		assert_eq!(Pools::<T>::get(pool_id).unwrap(), pool_info);
	}

	deposit_lp {
		let caller = funded_account::<T>("caller", 0);
		let alloc_point = 1000u128;
		let pool_id = T::PoolId::zero();
		let amount: Balance = 100_000_000_000_000;

		// create a pool
		TotalAllocPoint::<T>::put(alloc_point);
		let pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1);
		Pools::<T>::insert(pool_id, pool_info);

	}: _(RawOrigin::Signed(caller.clone()), pool_id, amount)
	verify {
		let mut pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1);
		pool_info.total_amount = amount;
		assert_eq!(Pools::<T>::get(pool_id).unwrap(), pool_info);

		let participant = Participant::new(amount, 0);
		assert_eq!(Participants::<T>::get(pool_id, caller).unwrap(), participant);
	}

	withdraw_lp {
		let caller = funded_account::<T>("caller", 0);
		let pool_id = T::PoolId::zero();
		let dico_per_block: Balance = 100_000_000_000_000;
		let amount: Balance = 100_000_000_000_000;

		HalvingPeriod::<T>::put(T::BlockNumber::from(5000u32));
		StartBlock::<T>::put(T::BlockNumber::from(1000u32));
		DicoPerBlock::<T>::put(dico_per_block);

		// create a pool
		let alloc_point = 1000u128;
		TotalAllocPoint::<T>::put(alloc_point);
		let pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1000);
		Pools::<T>::insert(pool_id, pool_info);

		Farm::<T>::deposit_lp(RawOrigin::Signed(caller.clone()).into(), pool_id, amount)?;

		// set system block number
		System::<T>::set_block_number(T::BlockNumber::from(16001u32));

	}: _(RawOrigin::Signed(caller.clone()), pool_id, 0)
	verify {
		let mut pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1);
		pool_info.total_amount = amount;
		pool_info.acc_dico_per_share = 8750125000000000u128;
		pool_info.last_reward_block = 16001;
		assert_eq!(Pools::<T>::get(pool_id).unwrap(), pool_info);

		let participant = Participant::new(amount, 875012500000000000);
		assert_eq!(Participants::<T>::get(pool_id, caller).unwrap(), participant);
	}
}
