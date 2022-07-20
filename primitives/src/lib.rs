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
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use cumulus_primitives_core::ParaId;

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

pub type CurrencyId = u32;
pub type Price = Balance;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;
