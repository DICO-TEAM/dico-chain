// This file is part of DICO.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Staking pallet benchmarking.
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_system::RawOrigin as SystemOrigin;
use frame_system::EventRecord;
use frame_benchmarking::{
	benchmarks_instance,
	account,
	benchmarks,
	whitelisted_caller,
	impl_benchmark_test_suite,
};
use sp_runtime::traits::Bounded;
use sp_std::mem::size_of;
use frame_system::Call as SystemCall;
use frame_system::Pallet as System;
use dico_primitives::parachains::native::LT::AssetId;
use ico::system::RawOrigin;
use crate::Module as Collective;
use ico::IcoParameters;
pub use crate::Pallet as Dao;

const SEED: u32 = 0;
const MAX_BYTES: u32 = 1_024;

fn get_vote<T: Config>(currency_id: u32, hash: T::Hash) -> IcoCollectiveVotes<T::AccountId, T::BlockNumber, MultiBalanceOf<T>> {
	Voting::<T>::get(currency_id, hash).unwrap()
}

fn get_alice<T: Config>() -> T::AccountId {
	account("alice", 1, SEED)
}

fn get_bob<T: Config>() -> T::AccountId {
	account("bob", 1, SEED)
}

fn set_propose<T: Config>() -> (T::Hash, u32, u32){
	let caller: T::AccountId = whitelisted_caller();
	let currency_id = 1;
	let ico_index = 1;
	T::IcoHandler::set_ico_for_bench(currency_id, ico_index, caller.clone(), get_alice::<T>(), get_bob::<T>());
	let proposal: T::Proposal = SystemCall::<T>::remark { remark: vec![1; 3] }.into();
	let hash = T::Hashing::hash_of(&proposal);

	assert!(Dao::<T>::propose(RawOrigin::Signed(get_alice::<T>()).into(), currency_id, ico_index,
							  Percent::from_percent(30u8), Box::new(proposal),
							  vec![1;3], 100).is_ok());

	(hash, currency_id, ico_index)
}

benchmarks! {
	propose {
		let caller: T::AccountId = whitelisted_caller();
		let proposal: T::Proposal = SystemCall::<T>::remark { remark: vec![1; 3] }.into();
		let hash = T::Hashing::hash_of(&proposal);
		let currency_id = 1;
		let ico_index = 1;
		T::IcoHandler::set_ico_for_bench(currency_id, ico_index, caller.clone(), get_alice::<T>(), get_bob::<T>());
	}:_(RawOrigin::Signed(get_alice::<T>()), currency_id, 1, Percent::from_percent(30u8), Box::new(proposal), vec![1;3], 100)

	vote {
		let caller: T::AccountId = get_bob::<T>();
		let (hash, currency_id, ico_index) = set_propose::<T>();
	}:_(RawOrigin::Signed(caller), currency_id, ico_index, hash, 0, true)

	close {
		let caller: T::AccountId = whitelisted_caller();
		let (hash, currency_id, ico_index) = set_propose::<T>();
		frame_system::Pallet::<T>::set_block_number(T::BlockNumber::from(1000000u32));
	}:_(RawOrigin::Signed(caller), currency_id, ico_index, hash, 0, 100, 100)

	disapprove_proposal {
		let caller: T::AccountId = get_alice::<T>();
		let (hash, currency_id, ico_index) = set_propose::<T>();

	}:_(RawOrigin::Root, currency_id, hash)

}

// impl_benchmark_test_suite!(
// 	Collective,
// 	crate::mock::new_test_ext(),
// 	crate::mock::Test,
// );

#[cfg(test)]
mod test1 {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {

			// assert_ok!(Dao::<Test>::test_benchmark_close());
			// assert_ok!(Dao::<Test>::test_benchmark_propose());
			// assert_ok!(Dao::<Test>::test_benchmark_vote());
			// assert_ok!(Dao::<Test>::test_benchmark_disapprove_proposal());
		});
	}
}
