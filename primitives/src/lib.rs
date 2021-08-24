#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::Member,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	DispatchResult, FixedU128, MultiSignature, OpaqueExtrinsic, Perbill, Permill, RuntimeDebug,
};
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use sp_version::NativeVersion;

pub mod asset;
pub mod constants;
pub use constants::*;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it
/// equivalent to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

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

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Signed version of Balance
pub type Amount = i128;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	BTC = 0,
	ETH = 1,
	DOT = 2,
}

pub type CurrencyId = u32;
pub type Price = FixedU128;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;
