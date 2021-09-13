//! Mocks for the amm module.

#![cfg(test)]

use crate as amm;
use crate::Config;
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::{H256};
use sp_runtime::{testing::Header, traits::{BlakeTwo256, IdentityLookup, Zero}};
use frame_support::traits::GenesisBuild;
use dico_primitives::{AssetId, Balance, BlockNumber};
use dico_currencies::BasicCurrencyAdapter;

pub type Amount = i128;
pub type AccountId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

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
		 AMM: amm::{Pallet, Call, Storage, Event<T>},
		 Tokens: orml_tokens::{Pallet, Event<T>},
		 Currency: dico_currencies::{Pallet, Event<T>, Call, Storage},
		 Balances: pallet_balances::{Pallet, Event<T>},
	 }
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 63;
	pub const AMMPalletId: PalletId = PalletId(*b"dico/amm");
	pub const AmmLiquidityAssetIdBase: AssetId = 20000000;
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
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type AccountStore = frame_system::Pallet<Test>;
	type WeightInfo = ();
	type ReserveIdentifier = ();
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

parameter_types! {
	pub const CreateConsume: Balance = 0;
	pub const DICOAssetId: AssetId = 0;
}

impl dico_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = DICOAssetId;
	type WeightInfo = ();
	type CreateConsume = CreateConsume;
}


impl Config for Test {
	type Event = Event;
	type LiquidityAssetIdBase = AmmLiquidityAssetIdBase;
	type Currency = Currency;
	type PalletId = AMMPalletId;
	type WeightInfo = ();
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
				// (ALICE, DICO, DEFAULT_ASSET_AMOUNT),
				// (BOB, DICO, DEFAULT_ASSET_AMOUNT),
				// (ALICE, USDT, DEFAULT_ASSET_AMOUNT),
				// (BOB, USDT, DEFAULT_ASSET_AMOUNT),
				// (ALICE, DOT, DEFAULT_ASSET_AMOUNT),
				// (BOB, DOT, DEFAULT_ASSET_AMOUNT),
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


