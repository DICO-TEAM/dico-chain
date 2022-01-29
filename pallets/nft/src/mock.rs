//! Mocks for the gradually-update module.

#![cfg(test)]

use super::*;
use crate as nft;
use frame_support::{construct_runtime, parameter_types};
use pallet_balances;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
type Balance = u64;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = u128;
pub type BlockNumber = u64;

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MaxTokenAttribute: u32 = 1024;
}

impl Config for Runtime {
	type WeightInfo = ();
	type ClassId = u64;
	type TokenId = u64;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
	type Event = Event;
	type Currency = Balances;
	type MaxTokenAttribute = MaxTokenAttribute;
	type PowerHandler = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		NonFungibleTokenModule: nft::{Pallet, Storage, Event<T>, Call},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: <Runtime as Config>::ClassId = 0;
pub const CLASS_ID_NOT_EXIST: <Runtime as Config>::ClassId = 100;
pub const TOKEN_ID: <Runtime as Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Runtime as Config>::TokenId = 100;

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();
	pallet_balances::GenesisConfig::<Runtime> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
