//! Mocks for the farm module.

#![cfg(test)]

use crate as farm;
use crate::Config;
use dico_primitives::{AssetId, Balance};
use frame_support::traits::{ConstU32, Contains, GenesisBuild};
use frame_support::{ord_parameter_types, parameter_types, PalletId};
use frame_system as system;
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};

pub type Amount = i128;
pub type AccountId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const DAVE: AccountId = 3;

pub const DICO: AssetId = 1000;
pub const DOT: AssetId = 2000;
// pub const KSM: AssetId = 3000;
pub const USDT: AssetId = 4000;
pub const PDOTUSDT: AssetId = 20000000;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 Farm: farm::{Pallet, Call, Storage, Event<T>},
		 Currency: orml_tokens::{Pallet, Event<T>},
	 }
);

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == DAVE
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub const FarmPalletId: PalletId = PalletId(*b"dico/fam");
	pub const DICOAssetId: AssetId = 0;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
	type DustRemovalWhitelist = MockDustRemovalWhitelist;
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
}
impl Config for Test {
	type Event = Event;
	type Currency = Currency;
	type PalletId = FarmPalletId;
	type NativeAssetId = DICOAssetId;
	type PoolId = u32;
	type WeightInfo = ();
	type FounderSetOrigin = EnsureSignedBy<One, u64>;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

pub const DEFAULT_ASSET_AMOUNT: Balance = 1000_000_000_000_000;

// Returns default values for genesis config
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, DICO, DEFAULT_ASSET_AMOUNT),
				(BOB, DICO, DEFAULT_ASSET_AMOUNT),
				(ALICE, USDT, DEFAULT_ASSET_AMOUNT),
				(BOB, USDT, DEFAULT_ASSET_AMOUNT),
				(ALICE, DOT, DEFAULT_ASSET_AMOUNT),
				(BOB, DOT, DEFAULT_ASSET_AMOUNT),
				(ALICE, PDOTUSDT, DEFAULT_ASSET_AMOUNT),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		orml_tokens::GenesisConfig::<Test> {
			balances: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
