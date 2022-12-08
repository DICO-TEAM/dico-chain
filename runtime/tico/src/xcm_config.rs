use super::*;
use crate::xcm_impls::FixedRateOfAsset;
use pallet_currencies::currencies_trait::AssetIdMapping;
use sp_runtime::WeakBoundedVec;

type WeakBoundedVec1 = WeakBoundedVec<u8, ConstU32<32>>;

pub fn ksm_per_second() -> u128 {
	let base_weight = Balance::from(ExtrinsicBaseWeight::get().ref_time());
	let base_tx_fee = DOLLARS / 1000;
	let base_tx_per_second = (WEIGHT_PER_SECOND.ref_time() as u128) / base_weight;
	let fee_per_second = base_tx_per_second * base_tx_fee;
	fee_per_second / 100
}

pub struct CurrencyIdConvert;

impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
	fn convert(id: CurrencyId) -> Option<MultiLocation> {
		if let Some(i) = pallet_currencies::AssetIdMaps::<Runtime>::get_multi_location(id) {
			return Some(i);
		}
		match id {
			KSM => Some(MultiLocation::parent()),
			KICO => Some(MultiLocation::new(
				1,
				X2(
					Parachain(ParachainInfo::parachain_id().into()),
					GeneralKey(WeakBoundedVec1::try_from(b"KICO".to_vec()).unwrap()),
				),
			)),
			DICO => Some(MultiLocation::new(
				1,
				X2(
					Parachain(ParachainInfo::parachain_id().into()),
					GeneralKey(WeakBoundedVec1::try_from(b"DICO".to_vec()).unwrap()),
				),
			)),
			KAR => Some(MultiLocation::new(
				1,
				X2(
					Parachain(paras::karura::ID),
					GeneralKey(WeakBoundedVec1::try_from(paras::karura::KAR_KEY.to_vec()).unwrap()),
				),
			)),
			AUSD => Some(MultiLocation::new(
				1,
				X2(
					Parachain(paras::karura::ID),
					GeneralKey(WeakBoundedVec1::try_from(paras::karura::AUSD_KEY.to_vec()).unwrap()),
				),
			)),
			LKSM => Some(MultiLocation::new(
				1,
				X2(
					Parachain(paras::karura::ID),
					GeneralKey(WeakBoundedVec1::try_from(paras::karura::LKSM_KEY.to_vec()).unwrap()),
				),
			)),

			parachains::listen::lt::ASSET_ID => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::listen::PARA_ID),
					GeneralKey(WeakBoundedVec1::try_from(parachains::listen::lt::TOKEN_SYMBOL.to_vec()).unwrap()),
				),
			)),

			parachains::listen::like::ASSET_ID => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::listen::PARA_ID),
					GeneralKey(WeakBoundedVec1::try_from(parachains::listen::like::TOKEN_SYMBOL.to_vec()).unwrap()),
				),
			)),

			_ => None,
		}
	}
}

impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<CurrencyId> {
		if let Some(l) = pallet_currencies::AssetIdMaps::<Runtime>::get_currency_id(location.clone()) {
			return Some(l);
		}

		match location {
			MultiLocation {
				parents: 1,
				interior: Here,
			} => Some(KSM),
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if ParaId::from(id) == ParachainInfo::parachain_id() && key == WeakBoundedVec1::try_from(b"KICO".to_vec()).unwrap() => Some(KICO),
			MultiLocation {
				parents: 0,
				interior: X1(GeneralKey(key)),
			} if key == WeakBoundedVec1::try_from(b"KICO".to_vec()).unwrap() => Some(KICO),

			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if ParaId::from(id) == ParachainInfo::parachain_id() && key == WeakBoundedVec1::try_from(b"DICO".to_vec()).unwrap() => Some(DICO),
			MultiLocation {
				parents: 0,
				interior: X1(GeneralKey(key)),
			} if key == WeakBoundedVec1::try_from(b"DICO".to_vec()).unwrap() => Some(DICO),

			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if id == paras::karura::ID && key == WeakBoundedVec1::try_from(paras::karura::AUSD_KEY.to_vec()).unwrap() => Some(AUSD),
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if id == paras::karura::ID && key == WeakBoundedVec1::try_from(paras::karura::KAR_KEY.to_vec()).unwrap() => Some(KAR),
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if id == paras::karura::ID && key == WeakBoundedVec1::try_from(paras::karura::LKSM_KEY.to_vec()).unwrap() => Some(LKSM),

			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if id == parachains::listen::PARA_ID && key == WeakBoundedVec1::try_from(parachains::listen::lt::TOKEN_SYMBOL.to_vec()).unwrap() => {
				Some(parachains::listen::lt::ASSET_ID)
			}

			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralKey(key)),
			} if id == parachains::listen::PARA_ID && key == WeakBoundedVec1::try_from(parachains::listen::like::TOKEN_SYMBOL.to_vec()).unwrap() => {
				Some(parachains::listen::like::ASSET_ID)
			}

			_ => None,
		}
	}
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(a: MultiAsset) -> Option<CurrencyId> {
		if let MultiAsset {
			id: AssetId::Concrete(id),
			fun: _,
		} = a
		{
			Self::convert(id)
		} else {
			None
		}
	}
}

pub struct AccountIdToMultiLocation;

impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account_id: AccountId) -> MultiLocation {
		X1(AccountId32 {
			network: NetworkId::Any,
			id: account_id.into(),
		})
		.into()
	}
}

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
	fn take_revenue(revenue: MultiAsset) {
		if let MultiAsset {
			id: Concrete(location),
			fun: Fungible(amount),
		} = revenue
		{
			if let Some(currency_id) = CurrencyIdConvert::convert(location) {
				// ensure KaruraTreasuryAccount have ed for all of the cross-chain asset.
				// Ignore the result.
				let _ = Currencies::deposit(currency_id, &TreasuryAccount::get(), amount);
			}
		}
	}
}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

parameter_types! {
	pub DicoTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Currencies,
	UnknownTokens,
	IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
	AccountId,
	LocationToAccountId,
	CurrencyId,
	CurrencyIdConvert,
	DepositToAlternative<DicoTreasuryAccount, Currencies, CurrencyId, AccountId, Balance>,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: u64 = 100_000_000;
}

match_type! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	// AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
	// ^^^ Parent and its exec plurality get free execution
);

parameter_types! {
	pub KsmPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), ksm_per_second());
	pub KicoPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(ParachainInfo::parachain_id().into()), GeneralKey(WeakBoundedVec1::try_from(b"KICO".to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 30
	);
	pub KicoPerSecondOfCanonicalLocation: (AssetId, u128) = (
		MultiLocation::new(
			0,
			X1(GeneralKey(WeakBoundedVec1::try_from(b"KICO".to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 30
	);
	pub DicoPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(ParachainInfo::parachain_id().into()), GeneralKey(WeakBoundedVec1::try_from(b"DICO".to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 30
	);
	pub DicoPerSecondOfCanonicalLocation: (AssetId, u128) = (
		MultiLocation::new(
			0,
			X1(GeneralKey(WeakBoundedVec1::try_from(b"DICO".to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 30
	);
	pub AusdPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(paras::karura::ID), GeneralKey(WeakBoundedVec1::try_from(paras::karura::AUSD_KEY.to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 100
	);
	pub KarPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(paras::karura::ID), GeneralKey(WeakBoundedVec1::try_from(paras::karura::KAR_KEY.to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 50
	);

	pub LIKEPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::listen::PARA_ID), GeneralKey(WeakBoundedVec1::try_from(parachains::listen::like::TOKEN_SYMBOL.to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 50
	);

	pub LTPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::listen::PARA_ID), GeneralKey(WeakBoundedVec1::try_from(parachains::listen::lt::TOKEN_SYMBOL.to_vec()).unwrap())),
		).into(),
		ksm_per_second() * 50
	);

	pub LKSMPerSecond: (AssetId, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(paras::karura::ID), GeneralKey(WeakBoundedVec1::try_from(paras::karura::LKSM_KEY.to_vec()).unwrap())),
		).into(),
		ksm_per_second()
	);

	pub BaseRate: u128 = ksm_per_second();

}

pub type Trader = (
	FixedRateOfFungible<KsmPerSecond, ToTreasury>,
	FixedRateOfFungible<KicoPerSecond, ToTreasury>,
	FixedRateOfFungible<KicoPerSecondOfCanonicalLocation, ToTreasury>,
	FixedRateOfFungible<DicoPerSecond, ToTreasury>,
	FixedRateOfFungible<DicoPerSecondOfCanonicalLocation, ToTreasury>,
	FixedRateOfFungible<AusdPerSecond, ToTreasury>,
	FixedRateOfFungible<KarPerSecond, ToTreasury>,
	FixedRateOfFungible<LKSMPerSecond, ToTreasury>,
	FixedRateOfFungible<LTPerSecond, ToTreasury>,
	FixedRateOfFungible<LIKEPerSecond, ToTreasury>,
	FixedRateOfAsset<BaseRate, ToTreasury, pallet_currencies::AssetIdMaps<Runtime>>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	type IsTeleporter = ();
	// Teleporting is disabled.
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
}

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const MaxInstructions: u32 = 100;
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

impl orml_unknown_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

pub type EnsureRootOrHalfCouncil = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1u32, 2u32>,
>;

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SovereignOrigin = EnsureRootOrHalfCouncil;
}

parameter_types! {
	pub const BaseXcmWeight: u64 =  100_000_000;

	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
	pub const MaxAssetsForTransfer: usize = 2;
}

parameter_type_with_key! {
	pub ParachainMinFee: |location: MultiLocation| -> Option<u128> {
		#[allow(clippy::match_ref_pats)] // false positive
		match (location.parents, location.first_interior()) {
			(1, Some(Parachain(1000))) => Some(4_000_000_000),
			_ => Some(u128::MAX),
		}
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = Everything;
	type ReserveProvider = AbsoluteReserveProvider;
}
