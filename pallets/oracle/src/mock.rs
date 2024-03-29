#![cfg(test)]

use super::*;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, GenesisBuild},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub use primitives::{CurrencyId, Price};
use std::cell::RefCell;

mod oracle {
	pub use super::super::*;
}

pub type AccountId = u128;
type Key = u32;
type Value = u32;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

thread_local! {
	static TIME: RefCell<u32> = RefCell::new(0);
}

pub struct Timestamp;
impl Time for Timestamp {
	type Moment = u32;

	fn now() -> Self::Moment {
		TIME.with(|v| *v.borrow())
	}
}

impl Timestamp {
	pub fn set_timestamp(val: u32) {
		TIME.with(|v| *v.borrow_mut() = val);
	}
}

parameter_types! {
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: u32 = 600;
	pub const RootOperatorAccountId: AccountId = 4;
	pub const MaxOracleSize: u32 = 5;
}

impl Config for Test {
	type Event = Event;
	type OnNewData = ();
	type CombineData = DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type OracleKey = Key;
	type OracleValue = Value;
	type RootOperatorAccountId = RootOperatorAccountId;
	type MaxOracleSize = MaxOracleSize;
	type WeightInfo = ();
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		ModuleOracle: oracle::{Pallet, Storage, Call, Event<T>},

	}
);

// This function basically just builds a genesis storage key/value store
// according to our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	// let _ = oracle::GenesisConfig::<Test> {
	// 	members: vec![1, 2, 3].into(),
	// 	phantom: Default::default(),
	// }
	// .assimilate_storage(&mut storage);

	let mut t: sp_io::TestExternalities = storage.into();

	t.execute_with(|| {
		Timestamp::set_timestamp(12345);
	});

	t
}
