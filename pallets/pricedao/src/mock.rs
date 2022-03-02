#![cfg(test)]

use super::{Balance, Config, CurrencyId, DataFeeder, DataProvider, FixedU128, Price, Zero};
use crate as pallet_price;
use frame_system::EnsureSignedBy;
use sp_core::H256;
use std::cell::RefCell;

use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	FixedPointNumber,
};

use dico_currencies::BasicCurrencyAdapter;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::Time,
	traits::{ConstU32, Contains, GenesisBuild},
	PalletId,
};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use primitives::AssetId;

pub type Amount = i128;
pub type AccountId = u128;
pub type BlockNumber = u64;
type Key = u32;
type Value = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const DAVE: AccountId = 3;
pub const EVE: AccountId = 4;
pub const GAVIN: AccountId = 30;

// Configure a mock runtime to test the pallet.
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		DicoOracle: pallet_oracle::{Pallet, Storage, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		PriceDao: pallet_price::{Pallet, Storage, Call, Event<T>},
		AMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
		Currency: dico_currencies::{Pallet, Event<T>, Call, Storage},
		Tokens: orml_tokens::{Pallet, Event<T>},

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
	pub const MaxOracleSize: u32 = 5;
	pub const ExistentialDeposit: Balance = 1;

	pub const FeedPledgedBalance: Balance = 50_000_000_000_000_000;   // 2000_000_000_000_000_000
	pub const TreasuryModuleId: PalletId = PalletId(*b"dico/tre");
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
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_oracle::Config for Test {
	type Event = Event;
	type OnNewData = ();
	type CombineData = pallet_oracle::DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type OracleKey = Key;
	type OracleValue = Value;
	type RootOperatorAccountId = GetRootOperatorAccountId;
	type MaxOracleSize = MaxOracleSize;
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
			1 => Some(100),
			2 => Some(50000),
			3 => Some(100),
			4 => Some(u128::zero()),
			_ => None,
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const AMMPalletId: PalletId = PalletId(*b"dico/amm");
	pub const AmmLiquidityAssetIdBase: AssetId = 20000000;
}

impl pallet_amm::Config for Test {
	type Event = Event;
	type LiquidityAssetIdBase = AmmLiquidityAssetIdBase;
	type Currency = Currency;
	type PalletId = AMMPalletId;
	type WeightInfo = ();
	type CurrenciesHandler = Currency;
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
	type MaxCreatableCurrencyId = AmmLiquidityAssetIdBase;
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
	type DustRemovalWhitelist = MockDustRemovalWhitelist;
}

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == DAVE
	}
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
}

pub const DEFAULT_BALANCE: Balance = 2000_000_000_000_000_000;
pub const DEFAULT_LOW_BALANCE: Balance = 40_000_000_000_000_000;

// Returns default values for genesis config
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, DEFAULT_BALANCE),
				(BOB, DEFAULT_BALANCE),
				(DAVE, DEFAULT_LOW_BALANCE),
				(EVE, DEFAULT_LOW_BALANCE),
			],
		}
	}
}

pub const DEFAULT_ASSET_AMOUNT: Balance = 1000_000_000_000_000;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = ExtBuilder::default().build();
	t.execute_with(|| {
		Timestamp::set_timestamp(12345);
		System::set_block_number(1);
	});
	t
}
