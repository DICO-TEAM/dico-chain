#![cfg_attr(not(feature = "std"), no_std)]
#![allow(missing_docs)]

use codec::{Decode, Encode};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	FixedU128, MultiAddress, MultiSignature, OpaqueExtrinsic, RuntimeDebug,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};


pub mod macros;
pub use macros::*;
pub mod constants;
pub mod tokens;
pub use constants::*;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it
/// equivalent to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// The type for looking up accounts. We don't expect more than 4 billion of
/// them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Core asset id
pub const CORE_ASSET_ID: AssetId = 0;

/// Type for storing the id of an asset.
pub type AssetId = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Signed version of Balance
pub type Amount = i128;

/// Farm pool id.
pub type PoolId = u32;

// /// Digest item type.
// pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

pub use cumulus_primitives_core::ParaId;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	BTC = 0,
	ETH = 1,
	DOT = 2,
}

pub type CurrencyId = u32;
// pub type Price = FixedU128;
pub type Price = Balance;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

///Network
#[cfg(feature = "std")]
pub mod network {
	#[cfg(feature = "std")]
	use serde_json::{map::Map, value::Value};
	// https://github.com/paritytech/ss58-registry/blob/main/ss58-registry.json
	pub const KICO_PREFIX: u16 = 42;
	pub const DICO_PREFIX: u16 = 42;

	// 	pub const DICO_REGISTRY: Properties = json!({
	// 		  "prefix": DICO_PREFIX,
	// 		  "network": "DICO",
	// 		  "displayName": "DICO",
	// 		  "symbols": ["DICO"],
	// 		  "decimals": [14],
	// 		  "standardAccount": "*25519",
	// 		  "website": "https://dico.io/"
	// 		})
	// 		.as_object()
	// 		.expect("Network properties are valid; qed")
	// 		.to_owned();
	//
	// 	pub const KICO_REGISTRY: Properties = json!({
	// 		  "prefix": KICO_PREFIX,
	// 		  "network": "KICO",
	// 		  "displayName": "KICO",
	// 		  "symbols": ["KICO"],
	// 		  "decimals": [14],
	// 		  "standardAccount": "*25519",
	// 		  "website": "https://dico.io/"
	// 		})
	// 		.as_object()
	// 		.expect("Network properties are valid; qed")
	// 		.to_owned();
	//

	pub fn get_properties(network_type: NetworkType) -> Map<String, Value> {
		let mut properties = Map::new();

		match network_type {
			NetworkType::KICO => {
				properties.insert("ss58Format".into(), Value::from(KICO_PREFIX));
				properties.insert("tokenSymbol".into(), "KICO".into());
			}
			NetworkType::DICO => {
				properties.insert("ss58Format".into(), Value::from(DICO_PREFIX));
				properties.insert("tokenSymbol".into(), "DICO".into());
			}
		}

		properties.insert("tokenDecimals".into(), 14.into());
		properties
	}

	#[derive(Clone, Copy)]
	pub enum NetworkType {
		KICO,
		DICO,
	}
}
