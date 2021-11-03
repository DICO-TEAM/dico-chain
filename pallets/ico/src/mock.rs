#![cfg(test)]

use super::*;
use pallet_pricedao;
use pallet_oracle;
use dico_treasury;
use pallet_randomness_collective_flip;
use frame_support::{construct_runtime, parameter_types, traits::{LockIdentifier, Time}, PalletId};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
use std::cell::RefCell;
use crate as ico;
use frame_system;
use pallet_balances;

use currencies::{self as dico_currencies, BasicCurrencyAdapter};

type Key = u32;
type Value = u128;
type AccountId = u128;
type Balance = u128;
type Amount = i128;
type CurrencyId = u32;
type BlockNumber = u32;
const DOLLARS: u128 = 1000;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Tokens: tokens::{Pallet, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		IcoTest: ico::{Pallet, Event<T>, Call, Storage},
		Currencies: dico_currencies::{Pallet, Event<T>, Call, Storage},
		Kyc: pallet_kyc::{Pallet, Call, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		PriceDao: pallet_pricedao::{Pallet, Call, Storage, Event<T>},
		AMM: pallet_amm::{Pallet, Call, Storage, Event<T>},
		DicoOracle: pallet_oracle::{Pallet, Storage, Call, Event<T>},
		DicoTreasury: dico_treasury::{Pallet, Call, Storage, Event<T>},
	}
);

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
	pub const DicoProposalBond: Balance = 100 * DOLLARS;
	pub const DicoSpendPeriod: BlockNumber = 7 * DAYS;
	pub const DicoTreasuryModuleId: PalletId = PalletId(*b"treasury");
}


impl dico_treasury::Config for Test {
	type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
	type PalletId = DicoTreasuryModuleId;
	type MultiCurrency = Currencies;
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type Event = Event;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type ProposalBond = DicoProposalBond;
	type SpendPeriod = DicoSpendPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const GetRootOperatorAccountId: AccountId = 4;
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: u32 = 600;
	pub const MaxOracleSize: u32 = 32;
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


parameter_types! {
	pub const LiquidityAssetIdBase: AssetId = 50000;
	pub const AMMPalletId: PalletId = PalletId(*b"dico/amm");
}

impl pallet_amm::Config for Test {
	type Event = Event;
	type LiquidityAssetIdBase = LiquidityAssetIdBase;
	type Currency = Currencies;
	type PalletId = AMMPalletId;
	type WeightInfo = ();
}

parameter_types! {
	pub const PledgedBalance: Balance = DOLLARS;
	pub const WithdrawExpirationPeriod: BlockNumber = 20;
}


impl pallet_pricedao::Config for Test {
	type Event = Event;
	type Source = DicoOracle;
	type FeedOrigin = frame_system::EnsureRoot<AccountId>;
	type UpdateOraclesStorgage = DicoOracle;
	type BaseCurrency = Balances;
	type PledgedBalance = PledgedBalance;
	type DicoTreasuryModuleId = DicoTreasuryModuleId;
	type WithdrawExpirationPeriod = WithdrawExpirationPeriod;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const BasicDeposit: Balance = DOLLARS;
	pub const ServiceDeposit: Balance = DOLLARS;
	pub const MaxIAS: u32 = 5;
	pub const MaxSwordHolder: u32 = 5;
	pub const KYCPalletId: PalletId = PalletId(*b"dico/kyc");

}

impl pallet_kyc::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Randomness = RandomnessCollectiveFlip;
	// pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
	type PalletId = KYCPalletId;
	type BasicDeposit = BasicDeposit;
	type ServiceDeposit = ServiceDeposit;
	type MaxIAS = MaxIAS;
	type MaxSwordHolder = MaxSwordHolder;
	type Slashed = ();
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type IASOrigin = frame_system::EnsureRoot<AccountId>;
	type SwordHolderOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
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

parameter_types! {
	pub const MinProportion: Percent = Percent::from_percent(20u8);
	pub const GetNativeCurrencyId: CurrencyId = 0;
	pub const IcoTotalReward: Balance = DOLLARS;
	pub const InitiatorPledge: Balance = DOLLARS;
	pub const RequestPledge: Balance = DOLLARS;
	pub const RequestExpire: u64 = 100;
	pub const InitiatorBond: Percent = Percent::from_percent(10u8);
	pub const TerminateProtectPeriod: Percent = Percent::from_percent(10u8);
	pub const ReleaseProtectPeriod: Percent = Percent::from_percent(10u8);
	pub const ChillDuration: BlockNumber = 5 * DAYS;
	pub const InviterRewardProportion: Percent = Percent::from_percent(10u8);
	pub const InviteeRewardProportion: Percent = Percent::from_percent(10u8);
	pub const UsdtCurrencyId: AssetId = 5;

}

impl Config for Test {
	type Event = Event;
	type PermitIcoOrigin = frame_system::EnsureRoot<AccountId>;
	type RejectIcoOrigin = frame_system::EnsureRoot<AccountId>;
	type PermitReleaseOrigin = frame_system::EnsureRoot<AccountId>;
	type TerminateIcoOrigin = frame_system::EnsureRoot<AccountId>;
	type OnSlash = ();
	type MultiCurrency = Tokens; //
	type NativeCurrency = Balances; //
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type InitiatorPledge = InitiatorPledge;
	type InitiatorBond = InitiatorBond;
	type RequestPledge = RequestPledge;
	type RequestExpire = RequestExpire;
	type CurrenciesHandler = Currencies;
	type IcoTotalReward = IcoTotalReward;

	type DicoTreasuryHandler = DicoTreasury;
	type TerminateProtectPeriod = TerminateProtectPeriod;
	type ReleaseProtectPeriod = ReleaseProtectPeriod;
	type ChillDuration = ChillDuration;
	type InviterRewardProportion = InviterRewardProportion;
	type InviteeRewardProportion = InviteeRewardProportion;
	type PriceData = PriceDao;
	type UsdtCurrencyId = UsdtCurrencyId;
	type KycHandler = Kyc;

}

parameter_types! {
	pub const MaxLocks: u32 = 50;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: u32| -> Balance {
		Zero::zero()
	};
}

impl tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
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
	pub const CreateConsume: Balance = DOLLARS;
	pub const MaxCreatableCurrencyId: AssetId = 10000;
}

impl dico_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type CreateConsume = CreateConsume;
	type MaxCreatableCurrencyId = MaxCreatableCurrencyId;
}

// pub fn new_test_ext1() -> sp_io::TestExternalities {
// 	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
//
// 	let _ = pallet_oracle::GenesisConfig::default().build_storage::<Test>().unwrap()
// 	.assimilate_storage(&mut storage);
//
// 	let mut t: sp_io::TestExternalities = storage.into();
//
// 	t.execute_with(|| {
// 		Timestamp::set_timestamp(12345);
// 	});
//
// 	t
// }

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| { System::set_block_number(1);
		Timestamp::set_timestamp(12345);
	});
	ext
}
