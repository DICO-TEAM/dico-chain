use cumulus_primitives_core::ParaId;
pub use dico_primitives::{constants::currency::*, network::*, AccountId, Balance, BlockNumber, Signature};
use hex_literal::hex;
use kico_runtime::{
	AuraConfig, AuraId, BalancesConfig, CollatorSelectionConfig, CouncilConfig, DemocracyConfig, ElectionsConfig,
	GenesisConfig, ParachainInfoConfig, Perbill, SessionConfig, SessionKeys, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, TechnicalMembershipConfig, VestingConfig, WASM_BINARY, CurrenciesConfig,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::AccountId32;
use sp_runtime::traits::{IdentifyAccount, Saturating, Verify, Zero};
use pallet_currencies::{DicoAssetMetadata, DicoAssetInfo};
use sp_core::crypto::Ss58Codec;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn kico_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"KICO",
		// ID
		"KICO",
		ChainType::Development,
		move || {
			let initial_authorities: Vec<(AccountId, AuraId)> = vec![
				(
					// 5Gc9gNiPUZ4KaZfkZRQUNWPudHhkKDSUVUXC3FEZNjHhtQ6H //collator1
					hex!["3ca222ba9c9cdae798cd40cf2d0523b248acd54b839465984f3c4854746575ef"].into(),
					hex!["3ca222ba9c9cdae798cd40cf2d0523b248acd54b839465984f3c4854746575ef"].unchecked_into(),
				),
				(
					// 5CXPMQvx4ggp7ZueYSZB72zxqrRrvfkHFQuFnG5tjBNXs2st//collator2
					hex!["90b8f45433cb1d85a9a8d7227aee0ad55a58fb25525ceec96449ab48d82e08d4"].into(),
					hex!["90b8f45433cb1d85a9a8d7227aee0ad55a58fb25525ceec96449ab48d82e08d4"].unchecked_into(),
				),
			];

			let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");

			let mut endowed_accounts: Vec<AccountId> = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			];

			let mut distribution_accounts: Vec<(AccountId, Balance)> = vec![
				// R&D: 500 million, 10% of Balance, 90% is vesting, 12 months linear distribution
				"5H9Kw8MJYNrXpRCNSxqQw8VwwtWvt5pP3wuLkv3mZnFUiWEU",
				// Foundation: 500 million, 10% of Balance, 90% is vesting, 24 months linear distribution
				"5EKpnRocPNmPgBtkML87eQyovEQYhF7ZFytCUavBgqDRY7H2",
				// Airdrop: 100 million, 100% of Balance
				"5EemocaRg4Q1fWskskZsYv5W2eTG3DDxmocg7QMLqHNUBwtX",
				// Market: 400 million, 10% of Balance, 90% is vesting, 12 months linear distribution
				"5DcKbSQMBWL9rNpu4hEEgHsdhBpBjW9AroN67szRJecMDSyK",
				// Aution:  400 million, 30% of Balance, 70% is vesting, 12 months linear distribution
				"5HTzXU1pYHD7YG55S2TqSeQLbM9cWvEyv3Ayqjf3j33nM8pV",
			]
			.iter()
			.flat_map(|x| {
				if x == &"5H9Kw8MJYNrXpRCNSxqQw8VwwtWvt5pP3wuLkv3mZnFUiWEU" {
					vec![(x.clone().parse().unwrap(), 500 * MILLIONDOLLARS)]
				} else if x == &"5EKpnRocPNmPgBtkML87eQyovEQYhF7ZFytCUavBgqDRY7H2" {
					vec![(x.clone().parse().unwrap(), 500 * MILLIONDOLLARS)]
				} else if x == &"5EemocaRg4Q1fWskskZsYv5W2eTG3DDxmocg7QMLqHNUBwtX" {
					vec![(x.clone().parse().unwrap(), 100 * MILLIONDOLLARS)]
				} else if x == &"5DcKbSQMBWL9rNpu4hEEgHsdhBpBjW9AroN67szRJecMDSyK" {
					vec![(x.clone().parse().unwrap(), 400 * MILLIONDOLLARS)]
				} else if x == &"5HTzXU1pYHD7YG55S2TqSeQLbM9cWvEyv3Ayqjf3j33nM8pV" {
					vec![(x.clone().parse().unwrap(), 400 * MILLIONDOLLARS)]
				} else {
					vec![(x.clone().parse().unwrap(), DOLLARS)]
				}
			})
			.collect();

			let initial_nominators: Vec<AccountId> = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
			];
			initial_authorities
				.iter()
				.map(|x| &x.0)
				.chain(initial_nominators.iter())
				.for_each(|x| {
					if !endowed_accounts.contains(&x) {
						endowed_accounts.push(x.clone())
					}
				});

			let council = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			];
			let technical_committee = vec![
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			];

			kico_genesis(
				root_key,
				initial_authorities,
				endowed_accounts,
				&mut distribution_accounts,
				council,
				technical_committee,
				id,
			)
		},
		vec![],
		TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.into(), 0)]).ok(),
		None,
		Some(get_properties(NetworkType::KICO)),
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: id.into(),
		},
	)
}

fn kico_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	distribution_accounts: &mut Vec<(AccountId, Balance)>,
	council: Vec<AccountId>,
	technical_committee: Vec<AccountId>,
	id: ParaId,
) -> kico_runtime::GenesisConfig {
	let num_endowed_accounts = endowed_accounts.len();
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
		serde_json::from_str(include_str!("../../../resources/kico-vesting-KICO.json")).unwrap();

	let mut balances = endowed_accounts
		.iter()
		.cloned()
		.map(|k| (k, 1 << 60))
		.collect::<Vec<(AccountId, Balance)>>();
	// .append(distribution_accounts);
	balances.append(distribution_accounts);

	GenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig { balances },
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: Zero::zero(),
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),          // account id
						acc.clone(),          // validator id
						SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},

		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		technical_committee: Default::default(),
		technical_membership: TechnicalMembershipConfig {
			members: technical_committee,
			phantom: Default::default(),
		},
		tokens: Default::default(),
		vesting: VestingConfig { vesting: vesting_list },
		currencies: CurrenciesConfig {
			assets: vec![(0, DicoAssetInfo {
				owner: AccountId32::from_string("5EKzRRVjvBvZfcRJPaHJCw2yecP9uQXcm6vqNcnMh6bCjpPe").unwrap(),
				metadata: Some(DicoAssetMetadata {
					name: "kico".into(),
					symbol: "KICO".into(),
					decimals: 14u8,
				})
			})]
		},
		treasury: Default::default(),
		council: CouncilConfig::default(),
		sudo: SudoConfig { key: root_key },
		parachain_info: ParachainInfoConfig { parachain_id: id },
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}
