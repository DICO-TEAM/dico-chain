pub use dico_primitives::{constants::currency::*, network::*, AccountId, Balance, BlockNumber, Signature};
use pallet_currencies::{DicoAssetInfo, DicoAssetMetadata};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::sr25519;
use sp_runtime::traits::Zero;
use tico_runtime::{
	AuraConfig, AuraId, BalancesConfig, CollatorSelectionConfig, CouncilConfig, CurrenciesConfig, DemocracyConfig,
	ElectionsConfig, GenesisConfig, ParachainInfoConfig, Perbill, SessionConfig, SessionKeys, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, TechnicalMembershipConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{get_account_id_from_seed, get_authority_keys_from_seed, Extensions, STAGING_TELEMETRY_URL};

pub const PARA_ID: u32 = 2017;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn tico_config() -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"DICO",
		// ID
		"DICO",
		ChainType::Development,
		move || {
			let initial_authorities: Vec<(AccountId, AuraId)> = vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
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
				// Foundation: 400 million, 10% of Balance, 90% is vesting, 24 months linear distribution
				"5EKpnRocPNmPgBtkML87eQyovEQYhF7ZFytCUavBgqDRY7H2",
				// Airdrop: 100 million, 100% of Balance
				"5EemocaRg4Q1fWskskZsYv5W2eTG3DDxmocg7QMLqHNUBwtX",
				// Market: 500 million, 10% of Balance, 90% is vesting, 12 months linear distribution
				"5DcKbSQMBWL9rNpu4hEEgHsdhBpBjW9AroN67szRJecMDSyK",
				// Aution:  500 million, 30% of Balance, 70% is vesting, 48 weeks linear distribution
				"5HTzXU1pYHD7YG55S2TqSeQLbM9cWvEyv3Ayqjf3j33nM8pV",
				// TICO testnet: faucet account
				"5EPCz3BnYokr1e2SWovjPbtSKh3SC7Wq5UmhJKrqA7UcZeY2",
				// root
				"5G6QA5eEfnNhuPDgosvEDYff4Rmd7xLvqwoFvCBTcoPLndke",
			]
			.iter()
			.flat_map(|x| {
				if x == &"5H9Kw8MJYNrXpRCNSxqQw8VwwtWvt5pP3wuLkv3mZnFUiWEU" {
					vec![(x.clone().parse().unwrap(), 500 * MILLIONDOLLARS)]
				} else if x == &"5EKpnRocPNmPgBtkML87eQyovEQYhF7ZFytCUavBgqDRY7H2" {
					vec![(x.clone().parse().unwrap(), 400 * MILLIONDOLLARS)]
				} else if x == &"5EemocaRg4Q1fWskskZsYv5W2eTG3DDxmocg7QMLqHNUBwtX" {
					vec![(x.clone().parse().unwrap(), 100 * MILLIONDOLLARS)]
				} else if x == &"5DcKbSQMBWL9rNpu4hEEgHsdhBpBjW9AroN67szRJecMDSyK" {
					vec![(x.clone().parse().unwrap(), 500 * MILLIONDOLLARS)]
				} else if x == &"5HTzXU1pYHD7YG55S2TqSeQLbM9cWvEyv3Ayqjf3j33nM8pV" {
					vec![(x.clone().parse().unwrap(), 500 * MILLIONDOLLARS)]
				} else if x == &"5EPCz3BnYokr1e2SWovjPbtSKh3SC7Wq5UmhJKrqA7UcZeY2" {
					vec![(x.clone().parse().unwrap(), 100 * MILLIONDOLLARS)]
				} else if x == &"5G6QA5eEfnNhuPDgosvEDYff4Rmd7xLvqwoFvCBTcoPLndke" {
					vec![(x.clone().parse().unwrap(), 100 * MILLIONDOLLARS)]
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

			tico_genesis(
				root_key,
				initial_authorities,
				endowed_accounts,
				&mut distribution_accounts,
				council,
				technical_committee,
			)
		},
		vec![],
		TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.into(), 0)]).ok(),
		Some("tico"),
		None,
		Some(get_properties(NetworkType::DICO)),
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	)
}

fn tico_genesis(
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	distribution_accounts: &mut Vec<(AccountId, Balance)>,
	council: Vec<AccountId>,
	technical_committee: Vec<AccountId>,
) -> GenesisConfig {
	let num_endowed_accounts = endowed_accounts.len();
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	let vesting_list: Vec<(AccountId, BlockNumber, BlockNumber, u32, Balance)> =
		serde_json::from_str(include_str!("../../../../resources/tico-vesting-DICO.json")).unwrap();

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
			assets: vec![(
				0,
				DicoAssetInfo {
					owner: "5EKzRRVjvBvZfcRJPaHJCw2yecP9uQXcm6vqNcnMh6bCjpPe".parse().unwrap(),
					metadata: Some(DicoAssetMetadata {
						name: "DICO".into(),
						symbol: "DICO".into(),
						decimals: 14u8,
					}),
				},
			)],
		},
		treasury: Default::default(),
		council: CouncilConfig::default(),
		sudo: SudoConfig {
			key: Some(root_key),
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}
