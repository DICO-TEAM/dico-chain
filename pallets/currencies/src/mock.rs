//! Mocks for the gradually-update module.
#![cfg(test)]
use super::*;
use crate as currencies;
use crate::Pallet as CurrenciesPallet;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, Contains},
};
use orml_tokens as tokens;
use orml_traits::parameter_type_with_key;
use pallet_balances;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

type Amount = i128;
type Balance = u64;
type CurrencyId = u32;

pub const Alice: AccountId = 1;
// type Cu

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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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
	pub const MaxClassMetadata: u32 = 1;
	pub const MaxTokenMetadata: u32 = 1;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: u32| -> Balance {
		Zero::zero()
	};
}

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		*a == Alice
	}
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const GetNativeCurrencyId: CurrencyId = 0;
	pub const CreateConsume: Balance = 100;
	pub const MaxCreatableCurrencyId: CurrencyId = 100;
}

impl tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = MockDustRemovalWhitelist;
}

impl Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type CreateConsume = CreateConsume;
	type MaxCreatableCurrencyId = MaxCreatableCurrencyId;
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
		Currencies: currencies::{ Pallet, Storage, Event<T>, Call },
		Tokens: tokens::{Pallet, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		// Timestamp::set_timestamp(12345);
	});
	ext
}

// pub struct ExtBuilder;
//
// impl Default for ExtBuilder {
// 	fn default() -> Self {
// 		ExtBuilder
// 	}
// }
//
// impl ExtBuilder {
// 	pub fn build(self) -> sp_io::TestExternalities {
// 		let t = frame_system::GenesisConfig::default()
// 			.build_storage::<Runtime>()
// 			.unwrap();
//
// 		let mut ext = sp_io::TestExternalities::new(t);
// 		ext.execute_with(|| System::set_block_number(1));
// 		ext
// 	}
// }
