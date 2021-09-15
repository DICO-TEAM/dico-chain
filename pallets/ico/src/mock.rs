#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, parameter_types};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
use crate as ico;
use frame_system;
use pallet_balances;

use currencies::{self as dico_currencies, BasicCurrencyAdapter};

type Balance = u64;
type Amount = i128;
type CurrencyId = u32;
type BlockNumber = u32;

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

	}
);

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
}

parameter_types! {
	pub const MinProportion: Percent = Percent::from_percent(20u8);
	pub const GetNativeCurrencyId: CurrencyId = 0;
	pub const IcoTotalReward: Balance = 100_00000_00000;
	pub const InitiatorPledge: Balance = 100_0000;
	pub const RequestPledge: Balance = 100_0000;
	pub const RequestExpire: u64 = 100;
	pub const NativeMultiple: IcoMultiple = IcoMultiple {
		numerator: 2u32,
		denominator: 1u32,
	};


}

impl Config for Test {
	type Event = Event;
	type PermitIcoOrigin = frame_system::EnsureRoot<u64>;
	type RejectIcoOrigin = frame_system::EnsureRoot<u64>;
	type PermitReleaseOrigin = frame_system::EnsureRoot<u64>;
	type TerminateIcoOrigin = frame_system::EnsureRoot<u64>;
	type MinProportion = MinProportion;
	type OnSlash = ();

	type MultiCurrency = Tokens; //
	type NativeCurrency = Balances; //

	type GetNativeCurrencyId = GetNativeCurrencyId;
	type InitiatorPledge = InitiatorPledge;
	type RequestPledge = RequestPledge;
	type RequestExpire = RequestExpire;
	type NativeMultiple = NativeMultiple;

	type CurrenciesHandler = Currencies;

	type IcoTotalReward = IcoTotalReward;
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

impl dico_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
