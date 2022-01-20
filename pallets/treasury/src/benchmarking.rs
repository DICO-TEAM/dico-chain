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

use frame_benchmarking::{account, benchmarks_instance, impl_benchmark_test_suite, benchmarks, whitelisted_caller};
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;

use crate::Module as Treasury;

const SEED: u32 = 0;


benchmarks! {
	propose_spend {
		let caller: T::AccountId = whitelisted_caller();
		T::
	}

}
impl_benchmark_test_suite!(Treasury, crate::tests::new_test_ext(), crate::tests::Test,);
