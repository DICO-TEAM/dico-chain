use cumulus_primitives_core::ParaId;
pub use dico_primitives::{constants::currency::*, network::*, AccountId, Balance, BlockNumber, Signature};
use kico_runtime::{
    AuraConfig, AuraId, BalancesConfig, CouncilConfig, SessionKeys, DemocracyConfig, ElectionsConfig, ParachainInfoConfig, Perbill,
    SessionConfig, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_telemetry::TelemetryEndpoints;


const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<kico_runtime::GenesisConfig, Extensions>;

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
        "kico node",
        // ID
        "kico-node",
        ChainType::Development,
        move || {
            let initial_authorities: Vec<(AccountId, AuraId)> = vec![
                (
                    // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//collator1
                    hex!["1a5dd54d1cef45e6140b54f3b83fdbbf41fec82645ad826d4f8cf106c88dd00e"].into(),
                    hex!["1a5dd54d1cef45e6140b54f3b83fdbbf41fec82645ad826d4f8cf106c88dd00e"]
                        .unchecked_into(),
                ),
                (
                    // 5E5BxCjexvzgH9LsYUzMjD6gJaWiKkmadvjsHFPZmrXrK7Rf//collator2
                    hex!["c6b255117d87f959c4e564888dc4987e0c3c35a60872a7fac4c38d771b39b70c"].into(),
                    hex!["c6b255117d87f959c4e564888dc4987e0c3c35a60872a7fac4c38d771b39b70c"]
                        .unchecked_into(),
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

            let initial_nominators: Vec<AccountId> = vec![
                get_account_id_from_seed::<sr25519::Public>("Bob"),
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

            testnet_genesis(root_key, initial_authorities, endowed_accounts, id)
        },
        vec![],
        TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.into(), 0)]).ok(),
        Some("kico"),
        Some(get_properties(NetworkType::KICO)),
        Extensions {
            relay_chain: "westend-local".into(),
            para_id: id.into(),
        },
    )
}

fn testnet_genesis(
    root_key: AccountId,
    initial_authorities: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> kico_runtime::GenesisConfig {

    let num_endowed_accounts = endowed_accounts.len();
    const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
    const STASH: Balance = ENDOWMENT / 1000;

    kico_runtime::GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
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
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
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
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        },
        technical_membership: Default::default(),
        treasury: Default::default(),
        council: CouncilConfig::default(),
        sudo: SudoConfig { key: root_key },
        parachain_info: ParachainInfoConfig { parachain_id: id },
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}
