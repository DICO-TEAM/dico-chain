#![cfg(test)]

use super::*;
use crate as pallet_price;
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	FixedPointNumber,
};
use std::cell::RefCell;

use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types, sp_runtime::ModuleId, traits::GenesisBuild,
};
use frame_system as system;

pub type AccountId = u128;
pub type BlockNumber = u64;
type Key = u32;
type Value = u32;

// Configure a mock runtime to test the pallet.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		DicoOracle: pallet_oracle::{Module, Storage, Call, Config<T>, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		PriceDao: pallet_price::{Module, Storage, Call, Config<T>, Event<T>},

	}
);

ord_parameter_types! {
	pub const One: AccountId = 1;

}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: u32 = 600;
	pub const GetRootOperatorAccountId: AccountId = 4;
	pub const ExistentialDeposit: Balance = 1;

	pub const FeedPledgedBalance: Balance = 500;
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"dico/tre");
	pub const withdrawExpirationPeriod: BlockNumber = 10;
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

impl system::Config for Test {
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
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl pallet_oracle::Config for Test {
	type Event = Event;
	type OnNewData = ();
	type CombineData = pallet_oracle::DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type OracleKey = Key;
	type OracleValue = Value;
	type RootOperatorAccountId = GetRootOperatorAccountId;
	type MaxOracleSize = u32;
	type WeightInfo = ();
}

impl Config for Test {
	type Event = Event;
	type Source = MockDataProvider;
	type FeedOrigin = EnsureSignedBy<One, AccountId>;
	type UpdateOraclesStorgage = DicoOracle;
	type BaseCurrency = Balances;
	type PledgedBalance = FeedPledgedBalance;
	type DicoTreasuryModuleId = TreasuryModuleId;
	type WithdrawExpirationPeriod = withdrawExpirationPeriod;
	type WeightInfo = ();
}

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
	fn get(currency_id: &CurrencyId) -> Option<Price> {
		match currency_id {
			1 => Some(FixedU128::saturating_from_rational(99, 100)),
			2 => Some(FixedU128::saturating_from_integer(50000)),
			3 => Some(FixedU128::saturating_from_integer(100)),
			4 => Some(FixedU128::zero()),
			_ => None,
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
}

pub const DEFAULT_BALANCE: Balance = 2000_000_000_000_000;

// Returns default values for genesis config
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, DEFAULT_BALANCE),
				(BOB, DEFAULT_BALANCE),
				(DAVE, DEFAULT_BALANCE),
				(EVE, DEFAULT_BALANCE),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_price::GenesisConfig::<Test> {
			members: vec![1, 2, 3].into(),
			phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u128;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
// This function basically just builds a genesis storage key/value store
// according to our desired mockup.
pub fn new_test_ext1() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let _ = pallet_price::GenesisConfig::<Test> {
		members: vec![1, 2, 3].into(),
		phantom: Default::default(),
	}
	.assimilate_storage(&mut storage);

	let mut t: sp_io::TestExternalities = storage.into();

	t.execute_with(|| {
		Timestamp::set_timestamp(12345);
	});

	t
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let _ = oracle::GenesisConfig::<Test> {
		members: vec![1, 2, 3].into(),
		phantom: Default::default(),
	}
	.assimilate_storage(&mut storage);

	let mut t: sp_io::TestExternalities = storage.into();

	t.execute_with(|| {
		Timestamp::set_timestamp(12345);
	});

	t
}
