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

//! Treasury pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Treasury;
use frame_benchmarking::{account, benchmarks, benchmarks_instance, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn get_alice<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	T::MultiCurrency::deposit(
		T::GetNativeCurrencyId::get(),
		&caller,
		(10000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	caller
}

fn get_bob<T: Config>() -> T::AccountId {
	let caller: T::AccountId = account("bob", 1, SEED);
	T::MultiCurrency::deposit(
		T::GetNativeCurrencyId::get(),
		&caller,
		(10000 * DOLLARS).saturated_into::<BalanceOf<T>>(),
	);
	caller
}

fn look_up<T: Config>(who: T::AccountId) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(who)
}

fn propose<T: Config>() -> u32 {
	let caller: T::AccountId = get_alice::<T>();
	let caller_cp = look_up::<T>(caller.clone());
	assert!(Treasury::<T>::propose_spend(
		RawOrigin::Signed(caller.clone()).into(),
		T::GetNativeCurrencyId::get(),
		BalanceOf::<T>::from(100u32),
		caller_cp
	)
	.is_ok());
	0
}

fn approve_propose<T: Config>() {
	let index = propose::<T>();
	assert!(Treasury::<T>::approve_proposal(T::ApproveOrigin::successful_origin(), index).is_ok());
}

benchmarks! {
	propose_spend {
		let caller: T::AccountId = get_alice::<T>();
	}:_(RawOrigin::Signed(caller.clone()), T::GetNativeCurrencyId::get(), BalanceOf::<T>::from(100u32), look_up::<T>(get_bob::<T>()))
	verify {
		assert_eq!(ProposalCount::<T>::get(), 1);
	}

	reject_proposal {
		let proposal_index = propose::<T>();
	}:_<T::Origin>(T::RejectOrigin::successful_origin(), 0)
	verify {
		assert!(!Proposals::<T>::contains_key(0));
	}

	approve_proposal {
		let proposal_index = propose::<T>();
	}:_<T::Origin>(T::ApproveOrigin::successful_origin(), 0)
	verify {
		assert!(Approvals::<T>::get().len() > 0);
	}

	spend_fund {
		let alice = get_alice::<T>();
		approve_propose::<T>();
	}:_(RawOrigin::Signed(alice.clone()))


}

#[cfg(test)]
mod test1 {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;
	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			get_alice::<Test>();
			// assert_ok!(Currencies::<Runtime>::test_benchmark_set_metadata());
			assert_ok!(Treasury::<Test>::test_benchmark_spend_fund());
		});
	}
}
// impl_benchmark_test_suite!(Treasury, crate::tests::new_test_ext(), crate::tests::Test,);
