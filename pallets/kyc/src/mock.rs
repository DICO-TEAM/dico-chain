// Tests for KYC Pallet
// run : cargo test -p pallet-kyc
use super::*;
use crate as pallet_kyc;

use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{EnsureOneOf},
};
use frame_support_test::TestRandomness;
use frame_system::{EnsureRoot, EnsureSignedBy};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub type AccountId = u64;
pub type Balance = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;
pub const EVE: AccountId = 5;

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
		KYC: pallet_kyc::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const KYCPalletId: PalletId = PalletId(*b"dico/kyc");
	pub const MaxIAS: u32 = 200;
	pub const MaxSwordHolder: u32 = 200;
	pub const KYCBasicDeposit: u32 = 100;
	pub const KYCServiceDeposit: u32 = 10000;
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
}

type EnsureOneOrRoot = EnsureOneOf<EnsureRoot<u64>, EnsureSignedBy<One, u64>>;
type EnsureTwoOrRoot = EnsureOneOf<EnsureRoot<u64>, EnsureSignedBy<Two, u64>>;
type EnsureThreeOrRoot = EnsureOneOf<EnsureRoot<u64>, EnsureSignedBy<Three, u64>>;

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type PalletId = KYCPalletId;
	type BasicDeposit = KYCBasicDeposit;
	type ServiceDeposit = KYCServiceDeposit;
	type MaxIAS = MaxIAS;
	type MaxSwordHolder = MaxSwordHolder;
	type Slashed = ();
	type Randomness = TestRandomness<Self>;
	type ForceOrigin = EnsureOneOrRoot;
	type IASOrigin = EnsureTwoOrRoot;
	type SwordHolderOrigin = EnsureThreeOrRoot;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 10000),
			(BOB, 10000),
			(CHARLIE, 10000),
			(DAVE, 10000),
			(EVE, 10000),
		],
	}
		.assimilate_storage(&mut t)
		.unwrap();
	t.into()
}