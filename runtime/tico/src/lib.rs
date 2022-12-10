#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use daos_square;
pub use dico_primitives::{
	constants::{currency::*, time::*},
	tokens::{AUSD, DICO, KAR, KICO, KSM, LKSM},
	AccountId, Address, Amount, Balance, BlockNumber, CurrencyId, Hash, Header, Index, Moment, ParaId, PoolId, Price,
	Signature,
};
use orml_traits::{
	create_median_value_data_provider, location::AbsoluteReserveProvider, parameter_type_with_key, DataFeeder,
	Happened, MultiCurrency,
};
pub use orml_xcm_support::{DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
use pallet_currencies::BasicCurrencyAdapter;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, BlockNumberProvider, Convert, Zero},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, Percent,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

// A few exports that help ease life for downstream crates.
use frame_support::{
	construct_runtime, match_type, parameter_types,
	traits::{
		ConstU32,
		Contains, EnsureOneOf, EqualPrivilegeOnly, Everything, LockIdentifier, Nothing, OnKilledAccount, OnNewAccount,
		OnRuntimeUpgrade, U128CurrencyToVote,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND, },
		ConstantMultiplier, DispatchClass, IdentityFee, Weight,
	},
	PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use frame_system::EnsureRoot;

pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

// Polkadot Imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

// XCM Imports
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom,
	EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, LocationInverter, ParentAsSuperuser, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};

pub use pallet_amm;
pub use pallet_farm;
pub use pallet_farm_extend;
pub use pallet_kyc;
pub use pallet_lbp;
pub use pallet_pricedao;
pub use pallet_vc;
pub use vc::*;

use crate::constants::*;
use crate::migrations::*;
use crate::parachains::*;
use pallet_farm_rpc_runtime_api as farm_rpc;

mod constants;
mod migrations;
mod parachains;
mod vc;
// mod weights;
mod xcm_config;
mod xcm_impls;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem, ()>;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("TICO"),
	impl_name: create_runtime_str!("TICO"),
	authoring_version: 1,
	spec_version: 2022121001,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND
	.saturating_div(2)
	.set_proof_size(cumulus_primitives_core::relay_chain::v2::MAX_POV_SIZE as u64);

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System, Utility
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		// Utility: pallet_utility::{Pallet, Call, Event} = 2,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 6,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 7,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 9,
		// Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 10,
		// Council,Membership
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 11,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 12,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 13,
		// Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
		// TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 15,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 16,

		// Governance
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 21,
		// Consensus
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 30,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 32,
		Aura: pallet_aura::{Pallet, Config<T>, Storage} = 33,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config, Storage} = 34,
		//  3rd Party
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 40,
		Vesting: orml_vesting::{Pallet, Storage, Call, Event<T>, Config<T>} = 41,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 42,
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 43,
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 44,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 50,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 51,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 52,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 53,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>} = 54,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 55,

		// vc
		CreateDao: daos_create_dao::{Pallet, Storage, Call, Event<T>} = 60,
		DaoSudo: daos_sudo::{Pallet, Storage, Call, Event<T>} = 61,
		DaoAgency: daos_agency::{Pallet, Origin<T>, Storage, Call, Event<T>} = 62,
		DoAs: daos_doas::{Pallet, Storage, Call, Event<T>} = 63,
		Vault: pallet_vc::{Pallet, Storage, Call, Event<T>} = 64,
		DaoSquare: daos_square::{Pallet, Storage, Call, Event<T>} = 65,
		// DaoEmergency: daos_emergency::{Pallet, Storage, Call, Event<T>} = 66,

		//local pallet
		Kyc: pallet_kyc::{Pallet, Call, Storage, Event<T>} = 70,

		DicoTreasury: pallet_dico_treasury::{Pallet, Call, Storage, Event<T>} = 71,
		Dao: pallet_dao::{Pallet, Origin<T>, Event<T>, Call, Storage}  = 72,
		Ico: pallet_ico::{Pallet, Event<T>, Call, Storage} = 73,

		AMM: pallet_amm::{Pallet, Call, Storage, Event<T>} = 80,

		Nft: pallet_nft::{Pallet, Call, Storage, Event<T>} = 81,
		LBP: pallet_lbp::{Pallet, Call, Storage, Event<T>} = 82,
		Farm: pallet_farm::{Pallet, Call, Storage, Event<T>} = 83,
		FarmExtend: pallet_farm_extend::{Pallet, Call, Storage, Event<T>}= 84,
		PriceDao: pallet_pricedao::{Pallet, Call, Storage, Event<T>} = 85,
		Currencies: pallet_currencies::{Pallet, Event<T>, Call, Storage, Config<T>} = 86,
		DicoOracle: pallet_oracle::<Instance1>::{Pallet, Storage, Call, Event<T>}= 87,
	}
);

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

pub struct NewAccount;
impl OnNewAccount<AccountId> for NewAccount {
	fn on_new_account(_who: &AccountId) {
		pallet_currencies::UsersNumber::<Runtime>::mutate(0 as CurrencyId, |i| {
			*i = i.saturating_add(1u32);
		});
	}
}

pub struct KilledAccount;
impl OnKilledAccount<AccountId> for KilledAccount {
	fn on_killed_account(_who: &AccountId) {
		pallet_currencies::UsersNumber::<Runtime>::mutate(0 as CurrencyId, |i| {
			*i = i.saturating_sub(1u32);
		});
	}
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = NewAccount;
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = KilledAccount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub MultisigDepositBase: Balance = deposit(1, 88);
	pub MultisigDepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * DOLLARS;
	pub const TransferFee: u128 = 1 * CENTS;
	pub const CreationFee: u128 = 1 * CENTS;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1 * MILLICENTS;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Preimages = ();
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>,
	>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3u32, 4u32>,
	>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 1u32>,
	>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2u32, 3u32>,
	>;
	type InstantOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1u32, 1u32>,
	>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2u32, 3u32>,
	>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1u32, 1u32>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type VoteLockingPeriod = EnactmentPeriod;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 1 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

parameter_types! {
	pub const CandidacyBond: Balance = 10 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 1 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;

impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1u32, 2u32>,
>;

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const MaxApprovals: u32 = 100;
	pub const ProposalBondMaximum: Balance = 100 * DOLLARS;
}

impl pallet_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3u32, 5u32>,
	>;
	type RejectOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1u32, 2u32>,
	>;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
	type ProposalBondMaximum = ProposalBondMaximum;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Preimages = ();
	type PalletsOrigin = OriginCaller;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
	pub const MaxAuthorities: u32 = 100_000;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"dico/pcs");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 1;
	pub const MaxInvulnerables: u32 = 100;
}

type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1u32, 2u32>,
>;

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = ();
}

parameter_types! {
	pub const KYCPalletId: PalletId = PalletId(*b"dico/kyc");
	pub const MaxIAS: u32 = 200;
	pub const MaxSwordHolder: u32 = 200;
	pub const KYCBasicDeposit: Balance = 100 * DOLLARS;
	pub const KYCServiceDeposit: Balance = 200000 * DOLLARS;
	pub const AmmPalletId: PalletId = PalletId(*b"dico/amm");
	pub const FarmPalletId: PalletId = PalletId(*b"dico/fam");
	pub const LBPPalletId: PalletId = PalletId(*b"dico/lbp");
	pub const FarmExtendPalletId: PalletId = PalletId(*b"dico/fme");
}

/// Configure the pallet template in pallets/template.
impl pallet_kyc::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type PalletId = KYCPalletId;
	type BasicDeposit = KYCBasicDeposit;
	type ServiceDeposit = KYCServiceDeposit;
	type MaxIAS = MaxIAS;
	type MaxSwordHolder = MaxSwordHolder;
	type Slashed = Treasury;
	type Randomness = RandomnessCollectiveFlip;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type IASOrigin = EnsureRootOrHalfCouncil;
	type SwordHolderOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = pallet_kyc::weights::DicoWeight<Runtime>;
}

impl pallet_amm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Currencies;
	type CurrenciesHandler = Currencies;
	type LiquidityAssetIdBase = MaxCreatableCurrencyId;
	type PalletId = AmmPalletId;
	type WeightInfo = pallet_amm::weights::DicoWeight<Runtime>;
}

impl pallet_lbp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Currencies;
	type PalletId = LBPPalletId;
	type LbpId = u32;
	type WeightInfo = pallet_lbp::weights::DicoWeight<Runtime>;
	type TreasuryHandler = DicoTreasury;
	type FounderSetOrigin = EnsureRootOrMoreThanHalfCouncil;
}

impl pallet_farm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PoolId = u32;
	type Currency = Currencies;
	type FounderSetOrigin = EnsureRootOrMoreThanHalfCouncil;
	type NativeAssetId = DICOAssetId;
	type PalletId = FarmPalletId;
	type WeightInfo = pallet_farm::weights::DicoWeight<Runtime>;
}

impl pallet_farm_extend::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Currencies;
	type PalletId = FarmExtendPalletId;
	type PoolExtendId = u32;
	type WeightInfo = pallet_farm_extend::weights::DicoWeight<Runtime>;
}

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const TokensMaxReserves: u32 = 50;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Zero::zero()
	};
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		get_all_module_accounts().contains(a)
	}
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		TreasuryPalletId::get().into_account_truncating(),
		KYCPalletId::get().into_account_truncating(),
		AmmPalletId::get().into_account_truncating(),
		FarmPalletId::get().into_account_truncating(),
		LBPPalletId::get().into_account_truncating(),
		FarmExtendPalletId::get().into_account_truncating(),
	]
}

pub struct OnNewTokenAccount;
impl Happened<(AccountId, CurrencyId)> for OnNewTokenAccount {
	fn happened(t: &(AccountId, CurrencyId)) {
		pallet_currencies::UsersNumber::<Runtime>::mutate(&t.1.clone(), |i| {
			*i = i.saturating_add(1u32);
		})
	}
}

pub struct OnKilledTokenAccount;
impl Happened<(AccountId, CurrencyId)> for OnKilledTokenAccount {
	fn happened(t: &(AccountId, CurrencyId)) {
		pallet_currencies::UsersNumber::<Runtime>::mutate(&t.1.clone(), |i| {
			*i = i.saturating_sub(1u32);
		});
	}
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CurrencyHooks = ();
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = MaxLocks;
	type MaxReserves = TokensMaxReserves;
	type ReserveIdentifier = [u8; 8];
	type DustRemovalWhitelist = DustRemovalWhitelist;
}

parameter_types! {
	pub MinVestedTransfer: Balance = 0;
	pub const MaxVestingSchedules: u32 = 200;
}

impl orml_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = frame_system::EnsureSigned<AccountId>;
	type WeightInfo = ();
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = frame_system::Pallet<Runtime>;
}

parameter_types! {
	pub const CreateConsume: Balance = 100 * DOLLARS;
	pub const DICOAssetId: CurrencyId = 0;
	pub const MaxCreatableCurrencyId: CurrencyId = 4_000_000_000;
}

impl pallet_currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;

	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type SetLocationOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type ForceSetLocationOrigin = EnsureRootOrHalfCouncil;

	type GetNativeCurrencyId = DICOAssetId;

	type WeightInfo = pallet_currencies::weights::DicoWeight<Runtime>;

	type CreateConsume = CreateConsume;
	type MaxCreatableCurrencyId = MaxCreatableCurrencyId;
	type USDCurrencyId = USDCurrencyId;
}

// price data
parameter_types! {
	pub const MaxOracleSize: u32 = 100;
	pub const MinimumCount: u32 = 3;  // todo: The minimum number is 3
	pub const ExpiresIn: Moment = 1000 * 60 * 20; // todo: 60 mins
	pub ZeroAccountId: AccountId = AccountId::from([0u8; 32]);
	pub const FeedPledgedBalance: Balance = 5000 * DOLLARS;  // todo : pledge 500 dico?
	pub const WithdrawExpirationPeriod: BlockNumber = 7 * DAYS;   // TODO: 5 * DAYS;
}

type DicoDataProvider = pallet_oracle::Instance1;

impl pallet_oracle::Config<DicoDataProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnNewData = ();
	type CombineData = pallet_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, DicoDataProvider>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type MaxOracleSize = MaxOracleSize;
	type RootOperatorAccountId = ZeroAccountId;
	type WeightInfo = pallet_oracle::weights::DicoWeight<Runtime>;
}

pub type TimeStampedPrice = pallet_oracle::TimestampedValue<Price, Moment>;
create_median_value_data_provider!(
	AggregatedDataProvider,
	CurrencyId,
	Price,
	TimeStampedPrice,
	[DicoOracle]
);

// Aggregated data provider cannot feed.
impl DataFeeder<CurrencyId, Price, AccountId> for AggregatedDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> DispatchResult {
		Err("Not supported".into())
	}
}

parameter_types! {
	pub const DicoTreasuryModuleId: PalletId = PalletId(*b"dico/trs");   // todo: modify name
}

type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1u32, 2u32>,
>;

impl pallet_pricedao::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Source = AggregatedDataProvider;
	type FeedOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type UpdateOraclesStorgage = DicoOracle;
	type DicoTreasuryModuleId = DicoTreasuryModuleId;
	type BaseCurrency = Balances;
	type PledgedBalance = FeedPledgedBalance;
	type WithdrawExpirationPeriod = WithdrawExpirationPeriod;
	type WeightInfo = pallet_pricedao::weights::DicoWeight<Runtime>;
}

parameter_types! {
	pub const DicoProposalBond: Balance = 100 * DOLLARS;
	pub const DicoSpendPeriod: BlockNumber = 7 * DAYS;

}

impl pallet_dico_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ApproveOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>,
	>;
	type PalletId = TreasuryPalletId;
	type MultiCurrency = Currencies;
	type RejectOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>,
	>;
	type GetNativeCurrencyId = DICOAssetId;
	type ProposalBond = DicoProposalBond;
	type SpendPeriod = DicoSpendPeriod;
	type WeightInfo = pallet_dico_treasury::weights::DicoWeight<Runtime>;
}

parameter_types! {
	pub const InitiatorPledge: Balance = 100 * DOLLARS;
	pub const RequestPledge: Balance = 300 * DOLLARS;
	pub const RequestExpire: BlockNumber = 5 * DAYS;
	pub const MinProportion: Percent = Percent::from_percent(0u8);
	pub const IcoTotalReward: Balance = 20_0000_0000 * DOLLARS;
	pub const InitiatorBond: Percent = Percent::from_percent(1u8);
	pub const TerminateProtectPeriod: Percent = Percent::from_percent(33);
	pub const ReleaseProtectPeriod: Percent = Percent::from_percent(33);
	pub const ChillDuration: BlockNumber = 10 * MINUTES;
	pub const InviterRewardProportion: Percent = Percent::from_percent(10u8);
	pub const InviteeRewardProportion: Percent = Percent::from_percent(5u8);
	pub const USDCurrencyId: CurrencyId = AUSD;
}

impl pallet_ico::Config for Runtime {
	type RuntimeEvent = RuntimeEvent ;
	type PermitIcoOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>;
	type RejectIcoOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>;
	type PermitReleaseOrigin = pallet_dao::EnsureProportionAtLeast<Runtime, AccountId, 1, 2>;
	type TerminateIcoOrigin = pallet_dao::EnsureProportionAtLeast<Runtime, AccountId, 1, 2>;
	type OnSlash = ();
	type MultiCurrency = Currencies;
	type NativeCurrency = Balances;
	type GetNativeCurrencyId = DICOAssetId;
	type InitiatorPledge = InitiatorPledge;
	type RequestPledge = RequestPledge;
	type RequestExpire = RequestExpire;
	type CurrenciesHandler = Currencies;
	type IcoTotalReward = IcoTotalReward;
	type DicoTreasuryHandler = DicoTreasury;
	type InitiatorBond = InitiatorBond;
	type TerminateProtectPeriod = TerminateProtectPeriod;
	type ReleaseProtectPeriod = ReleaseProtectPeriod;
	type ChillDuration = ChillDuration;
	type InviterRewardProportion = InviterRewardProportion;
	type InviteeRewardProportion = InviteeRewardProportion;
	type PriceData = PriceDao;
	type KycHandler = Kyc;
	type WeightInfo = pallet_ico::weights::DicoWeight<Runtime>;
	type USDCurrencyId = USDCurrencyId;
}

parameter_types! {
	pub const DicoMotionDuration: BlockNumber = 5 * DAYS;
	pub const DicoMaxProposals: u32 = 100;
}

impl pallet_dao::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Proposal = RuntimeCall;
	type MotionDuration = DicoMotionDuration;
	type MaxProposals = DicoMaxProposals;
	type WeightInfo = pallet_dao::weights::DicoWeight<Runtime>;
	type IcoHandler = Ico;
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MaxTokenAttribute: u32 = 1024;

}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ClassId = u32;
	type TokenId = u32;
	type Currency = Balances;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
	type MaxTokenAttribute = MaxTokenAttribute;
	type PowerHandler = Ico;
	type USDCurrencyId = USDCurrencyId;
	type WeightInfo = pallet_nft::weights::DicoWeight<Runtime>;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl farm_rpc::FarmApi<Block, AccountId, PoolId, Balance> for Runtime {
		fn get_participant_reward(account: AccountId, pid: PoolId) -> Balance {
			let reward = Farm::get_participant_reward(account, pid);

			reward
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}


	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl pallet_ico_rpc_runtime_api::IcoAmountApi<Block, AccountId, CurrencyId, Index, Balance> for Runtime {
		fn can_release_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::can_release_amount(account, currency_id, index)
		}
		fn get_reward_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::get_reward_amount(account, currency_id, index)
		}
		fn can_unlock_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::can_unlock_amount(account, currency_id, index)
		}
		fn can_join_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> (Balance, Balance) {
			Ico::can_join_amount(account, currency_id, index)
		}

		fn get_token_price(currency_id: CurrencyId) -> Balance {
			Ico::get_token_price(currency_id)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade parachain-template.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block_no_check(block: Block) -> Weight {
			Executive::execute_block_no_check(block)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_multisig, Multisig);
			// list_benchmark!(list, extra, pallet_membership, TechnicalMembership);
			list_benchmark!(list, extra, pallet_collator_selection, CollatorSelection);
			// list_benchmark!(list, extra, pallet_amm, AMM);
			// list_benchmark!(list, extra, pallet_farm, Farm);
			// list_benchmark!(list, extra, pallet_lbp, LBP);
			// list_benchmark!(list, extra, pallet_farm_extend, FarmExtend);
			// list_benchmark!(list, extra, pallet_pricedao, PriceDao);
			// list_benchmark!(list, extra, pallet_oracle, DicoOracle);
			//
			//
			// list_benchmark!(list, extra, pallet_nft, Nft);
			// list_benchmark!(list, extra, pallet_kyc, Kyc);
			// list_benchmark!(list, extra, pallet_dao, Dao);
			// list_benchmark!(list, extra, pallet_ico, Ico);
			// list_benchmark!(list, extra, pallet_dico_treasury, DicoTreasury);
			// list_benchmark!(list, extra, pallet_currencies, Currencies);

			list_benchmark!(list, extra, daos_create_dao, CreateDao);
			list_benchmark!(list, extra, daos_agency, DaoAgency);
			list_benchmark!(list, extra, daos_sudo, DaoSudo);
			list_benchmark!(list, extra, daos_square, DaoSquare);
			list_benchmark!(list, extra, daos_doas, DoAs);
			// list_benchmark!(list, extra, daos_emergency, DaoEmergency);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_multisig, Multisig);
			// add_benchmark!(params, batches, pallet_membership, TechnicalMembership);
			// add_benchmark!(params, batches, pallet_amm, AMM);
			// add_benchmark!(params, batches, pallet_farm, Farm);
			// add_benchmark!(params, batches, pallet_lbp, LBP);
			// add_benchmark!(params, batches, pallet_farm_extend, FarmExtend);
			//
			// add_benchmark!(params, batches, pallet_kyc, Kyc);
			// add_benchmark!(params, batches, pallet_nft, Nft);
			// add_benchmark!(params, batches, pallet_dao, Dao);
			// add_benchmark!(params, batches, pallet_dico_treasury, DicoTreasury);
			// add_benchmark!(params, batches, pallet_ico, Ico);
			// add_benchmark!(params, batches, pallet_pricedao, PriceDao);
			// add_benchmark!(params, batches, pallet_oracle, DicoOracle);
			// add_benchmark!(params, batches, pallet_currencies, Currencies);

			add_benchmark!(params, batches, daos_create_dao, CreateDao);
			add_benchmark!(params, batches, daos_agency, DaoAgency);
			add_benchmark!(params, batches, daos_sudo, DaoSudo);
			// add_benchmark!(params, batches, daos_emergency, DaoEmergency);
			add_benchmark!(params, batches, daos_square, DaoSquare);
			add_benchmark!(params, batches, daos_doas, DoAs);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
