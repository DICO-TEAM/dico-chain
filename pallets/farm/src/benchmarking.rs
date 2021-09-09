#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use sp_std::prelude::*;
use frame_support::traits::UnfilteredDispatchable;
use sp_core::{U256};
use frame_system::{Pallet as System, RawOrigin};

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
        let origin = T::FounderSetOrigin::successful_origin();
        let call = Call::<T>::set_halving_period(T::BlockNumber::from(1000u32));

    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(HalvingPeriod::<T>::get(), T::BlockNumber::from(1000u32));
    }

    set_dico_per_block {
        let origin = T::FounderSetOrigin::successful_origin();
        let amount: Balance = 10000;
        let call = Call::<T>::set_dico_per_block(amount);

    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(DicoPerBlock::<T>::get(), amount);
    }

    set_start_block {
        let origin = T::FounderSetOrigin::successful_origin();
        let call = Call::<T>::set_start_block(T::BlockNumber::from(10000u32));

    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(StartBlock::<T>::get(), T::BlockNumber::from(10000u32));
    }

    create_pool {
        let origin = T::FounderSetOrigin::successful_origin();
        let alloc_point = U256::from(1000);
        let call = Call::<T>::create_pool(LIQUIDITY_ID, alloc_point);

    }: { call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(TotalAllocPoint::<T>::get(), alloc_point);
        let pool_info = PoolInfo::new(LIQUIDITY_ID, alloc_point, 1);
        assert_eq!(Pools::<T>::get(T::PoolId::zero()).unwrap(), pool_info);
    }

    update_pool_alloc_point {
        let origin = T::FounderSetOrigin::successful_origin();

        let initial_alloc_point = U256::from(1000);
        let update_alloc_point = U256::from(10000);
        let pool_id = T::PoolId::zero();

        let create_pool_call = Call::<T>::create_pool(LIQUIDITY_ID, initial_alloc_point);

        let call = Call::<T>::update_pool_alloc_point(pool_id, update_alloc_point);

    }: { create_pool_call.dispatch_bypass_filter(origin.clone())?; call.dispatch_bypass_filter(origin)? }
    verify {
        assert_eq!(TotalAllocPoint::<T>::get(), update_alloc_point);
        let pool_info = PoolInfo::new(LIQUIDITY_ID, update_alloc_point, 1);
        assert_eq!(Pools::<T>::get(pool_id).unwrap(), pool_info);
    }

    deposit_lp {
        let caller = funded_account::<T>("caller", 0);
        let alloc_point = U256::from(1000);
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
        assert_eq!(Users::<T>::get(pool_id, caller).unwrap(), participant);
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
        let alloc_point = U256::from(1000);
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
        pool_info.acc_dico_per_share = U256::from(8750125000000000u64);
        pool_info.last_reward_block = 16001;
        assert_eq!(Pools::<T>::get(pool_id).unwrap(), pool_info);

        let participant = Participant::new(amount, 875012500000000000);
        assert_eq!(Users::<T>::get(pool_id, caller).unwrap(), participant);
    }
}

