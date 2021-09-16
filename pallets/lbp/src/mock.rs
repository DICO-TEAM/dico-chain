//! Mocks for the lbp module.

#![cfg(test)]

use crate as lbp;
use crate::Config;
use frame_support::{parameter_types, PalletId, ord_parameter_types};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::{H256};
use sp_runtime::{testing::Header, traits::{BlakeTwo256, IdentityLookup, Zero}};
use frame_support::traits::GenesisBuild;
use dico_primitives::{AssetId, Balance};
use pallet_dico_treasury::traits::DicoTreasuryHandler;

pub type Amount = i128;
pub type AccountId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const TREASURY_ACCOUNT: AccountId = 10;

pub const DICO: AssetId = 1000;
pub const DOT: AssetId = 2000;
// pub const KSM: AssetId = 3000;
pub const USDT: AssetId = 4000;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
	 Block = Block,
	 NodeBlock = Block,
	 UncheckedExtrinsic = UncheckedExtrinsic,
	 {
		 System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		 Lbp: lbp::{Pallet, Call, Storage, Event<T>},
		 Currency: orml_tokens::{Pallet, Event<T>},
	 }
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub const LbpPalletId: PalletId = PalletId(*b"ico/lbpx");
	pub const DICOAssetId: AssetId = 0;
}

impl system::Config for Test {
	type BaseCallFilter = ();
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
	type DustRemovalWhitelist = ();
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
}

pub struct Treasury();
impl DicoTreasuryHandler<AccountId> for Treasury {
	fn get_treasury_account_id() -> AccountId {
		TREASURY_ACCOUNT
	}
}

impl Config for Test {
	type Event = Event;
	type Currency = Currency;
	type PalletId = LbpPalletId;
	type LbpId = u32;
	type WeightInfo = ();
	type TreasuryHandler = Treasury;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, AssetId, Balance)>,
}

pub const DEFAULT_ASSET_AMOUNT: Balance = 10_000_000_000_000_000_000_000_000_000;
pub const WEIGHT_ONE: u128 = 10_000_000_000u128;

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
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		orml_tokens::GenesisConfig::<Test> {
			balances: self.endowed_accounts,
		}.assimilate_storage(&mut t).unwrap();

		t.into()
	}
}
