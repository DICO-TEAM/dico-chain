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

//! Treasury pallet tests.

#![cfg(test)]

use super::*;
use crate as treasury;
use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{Contains, OnInitialize},
	PalletId,
};
use orml_tokens;
use orml_traits::parameter_type_with_key;
use pallet_currencies::BasicCurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use std::cell::RefCell;

pub type Balance = u64;
pub type Amount = i128;
pub type BlockNumber = u32;
type CurrencyId = u32;
pub type AccountId = u128;
pub const Alice: u128 = 1;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Treasury: treasury::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Config<T>, Storage, Event<T>},
		Currencies: pallet_currencies::{Pallet, Event<T>, Call, Storage},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}
parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}
thread_local! {
	static TEN_TO_FOURTEEN: RefCell<Vec<u128>> = RefCell::new(vec![10,11,12,13,14]);
}
parameter_types! {
	pub const ProposalBond: Balance = 1;
	pub const ProposalBondMinimum: Balance = 1;
	pub const SpendPeriod: Balance = 2;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BountyUpdatePeriod: u32 = 20;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 1;
}
parameter_types! {
	pub const MaxLocks: u32 = 100;
}

impl Config for Test {
	type PalletId = TreasuryPalletId;
	type MultiCurrency = Currencies;
	type ApproveOrigin = frame_system::EnsureRoot<u128>;
	type RejectOrigin = frame_system::EnsureRoot<u128>;
	type Event = Event;
	type ProposalBond = ProposalBond;
	type SpendPeriod = SpendPeriod;
	type WeightInfo = ();
	type GetNativeCurrencyId = GetNativeCurrencyId;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: u32| -> Balance {
		Zero::zero()
	};
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
	pub const CreateConsume: Balance = 10;
	pub const MaxCreatableCurrencyId: u32 = 100;
}
impl pallet_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type CreateConsume = CreateConsume;
	type MaxCreatableCurrencyId = MaxCreatableCurrencyId;
}

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == Alice
	}
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = MockDustRemovalWhitelist;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

// #[test]
// fn genesis_config_works() {
// 	new_test_ext().execute_with(|| {
// 		assert_eq!(Treasury::pot(), 0);
// 		assert_eq!(Treasury::proposal_count(), 0);
// 	});
// }
//
// #[test]
// fn minting_works() {
// 	new_test_ext().execute_with(|| {
// 		// Check that accumulate works when we have Some value in Dummy already.
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
// 	});
// }
//
// #[test]
// fn spend_proposal_takes_min_deposit() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 1, 3));
// 		assert_eq!(Balances::free_balance(0), 99);
// 		assert_eq!(Balances::reserved_balance(0), 1);
// 	});
// }
//
// #[test]
// fn spend_proposal_takes_proportional_deposit() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_eq!(Balances::free_balance(0), 95);
// 		assert_eq!(Balances::reserved_balance(0), 5);
// 	});
// }
//
// #[test]
// fn spend_proposal_fails_when_proposer_poor() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::propose_spend(Origin::signed(2), 100, 3),
// 			Error::<Test, _>::InsufficientProposersBalance,
// 		);
// 	});
// }
//
// #[test]
// fn accepted_spend_proposal_ignored_outside_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(1);
// 		assert_eq!(Balances::free_balance(3), 0);
// 		assert_eq!(Treasury::pot(), 100);
// 	});
// }
//
// #[test]
// fn unused_pot_should_diminish() {
// 	new_test_ext().execute_with(|| {
// 		let init_total_issuance = Balances::total_issuance();
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
// 		assert_eq!(Balances::total_issuance(), init_total_issuance + 100);
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 50);
// 		assert_eq!(Balances::total_issuance(), init_total_issuance + 50);
// 	});
// }
//
// #[test]
// fn rejected_spend_proposal_ignored_on_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Balances::free_balance(3), 0);
// 		assert_eq!(Treasury::pot(), 50);
// 	});
// }
//
// #[test]
// fn reject_already_rejected_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
// 		assert_noop!(
// 			Treasury::reject_proposal(Origin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }
//
// #[test]
// fn reject_non_existent_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::reject_proposal(Origin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }
//
// #[test]
// fn accept_non_existent_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Treasury::approve_proposal(Origin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }
//
// #[test]
// fn accept_already_rejected_spend_proposal_fails() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
// 		assert_noop!(
// 			Treasury::approve_proposal(Origin::root(), 0),
// 			Error::<Test, _>::InvalidIndex
// 		);
// 	});
// }
//
// #[test]
// fn accepted_spend_proposal_enacted_on_spend_period() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Balances::free_balance(3), 100);
// 		assert_eq!(Treasury::pot(), 0);
// 	});
// }
//
// #[test]
// fn pot_underflow_should_not_diminish() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 150, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 100); // Pot hasn't changed
//
// 		let _ = Balances::deposit_into_existing(&Treasury::account_id(), 100).unwrap();
// 		<Treasury as OnInitialize<u64>>::on_initialize(4);
// 		assert_eq!(Balances::free_balance(3), 150); // Fund has been spent
// 		assert_eq!(Treasury::pot(), 25); // Pot has finally changed
// 	});
// }
//
// // Treasury account doesn't get deleted if amount approved to spend is all its
// // free balance. i.e. pot should not include existential deposit needed for
// // account survival.
// #[test]
// fn treasury_account_doesnt_get_deleted() {
// 	new_test_ext().execute_with(|| {
// 		Balances::make_free_balance_be(&Treasury::account_id(), 101);
// 		assert_eq!(Treasury::pot(), 100);
// 		let treasury_balance = Balances::free_balance(&Treasury::account_id());
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), treasury_balance, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 100); // Pot hasn't changed
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), Treasury::pot(), 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 1));
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(4);
// 		assert_eq!(Treasury::pot(), 0); // Pot is emptied
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 1); // but the account is still there
// 	});
// }
//
// // In case treasury account is not existing then it works fine.
// // This is useful for chain that will just update runtime.
// #[test]
// fn inexistent_account_works() {
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	pallet_balances::GenesisConfig::<Test> {
// 		balances: vec![(0, 100), (1, 99), (2, 1)],
// 	}
// 	.assimilate_storage(&mut t)
// 	.unwrap();
// 	// Treasury genesis config is not build thus treasury account does not exist
// 	let mut t: sp_io::TestExternalities = t.into();
//
// 	t.execute_with(|| {
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 0); // Account does not exist
// 		assert_eq!(Treasury::pot(), 0); // Pot is empty
//
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 99, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
// 		assert_ok!(Treasury::propose_spend(Origin::signed(0), 1, 3));
// 		assert_ok!(Treasury::approve_proposal(Origin::root(), 1));
// 		<Treasury as OnInitialize<u64>>::on_initialize(2);
// 		assert_eq!(Treasury::pot(), 0); // Pot hasn't changed
// 		assert_eq!(Balances::free_balance(3), 0); // Balance of `3` hasn't changed
//
// 		Balances::make_free_balance_be(&Treasury::account_id(), 100);
// 		assert_eq!(Treasury::pot(), 99); // Pot now contains funds
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), 100); // Account does exist
//
// 		<Treasury as OnInitialize<u64>>::on_initialize(4);
//
// 		assert_eq!(Treasury::pot(), 0); // Pot has changed
// 		assert_eq!(Balances::free_balance(3), 99); // Balance of `3` has changed
// 	});
// }
//
// #[test]
// fn genesis_funding_works() {
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	let initial_funding = 100;
// 	pallet_balances::GenesisConfig::<Test> {
// 		// Total issuance will be 200 with treasury account initialized with 100.
// 		balances: vec![(0, 100), (Treasury::account_id(), initial_funding)],
// 	}
// 	.assimilate_storage(&mut t)
// 	.unwrap();
// 	treasury::GenesisConfig::default()
// 		.assimilate_storage::<Test, _>(&mut t)
// 		.unwrap();
// 	let mut t: sp_io::TestExternalities = t.into();
//
// 	t.execute_with(|| {
// 		assert_eq!(Balances::free_balance(Treasury::account_id()), initial_funding);
// 		assert_eq!(Treasury::pot(), initial_funding - Balances::minimum_balance());
// 	});
// }
