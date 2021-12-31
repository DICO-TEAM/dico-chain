use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use dico_primitives::{Signature, AccountId};

pub mod kico;
pub mod tico;


const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";



/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate authority keys from seed.
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AuraId) {
    (
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<AuraId>(seed),
    )
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
