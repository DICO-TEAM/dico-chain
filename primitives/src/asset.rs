use crate::{AssetId, Balance, BlockNumber};
use codec::{Decode, Encode};
use sp_std::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::U256;
use sp_runtime::RuntimeDebug;

/// Arithmetic errors.
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

#[macro_export]
macro_rules! to_u256 {
    ($($x:expr),+) => (
        {($(U256::from($x)),+)}
    );
}

#[macro_export]
macro_rules! to_balance {
	($x:expr) => {
		Balance::try_from($x).map_err(|_| ArithmeticError::Overflow)
	};
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SwapPair(pub AssetId, pub AssetId);

impl SwapPair {
	pub fn new(asset_a: AssetId, asset_b: AssetId) -> Self {
		if asset_a < asset_b {
			Self(asset_a, asset_b)
		} else {
			Self(asset_b, asset_a)
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PoolUserInfo {
	pub amount: Balance,
	pub reward_debt: Balance,
}

impl PoolUserInfo {
	pub fn new(amount: Balance, reward_debt: Balance) -> Self {
		Self { amount, reward_debt }
	}
}

impl Default for PoolUserInfo {
	fn default() -> Self {
		Self {
			amount: 0,
			reward_debt: 0,
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PoolInfo {
	pub liquidity_id: AssetId,
	pub alloc_point: U256,
	pub last_reward_block: BlockNumber,
	pub acc_dico_per_share: U256,
	pub total_amount: Balance,
}

impl PoolInfo {
	pub fn new(liquidity_id: AssetId, alloc_point: U256, last_reward_block: BlockNumber) -> Self {
		Self {
			liquidity_id,
			alloc_point,
			last_reward_block,
			acc_dico_per_share: U256::from(0),
			total_amount: 0,
		}
	}
}

impl Default for PoolInfo {
	fn default() -> Self {
		Self {
			liquidity_id: 0,
			alloc_point: U256::from(0),
			last_reward_block: 0,
			acc_dico_per_share: U256::from(0),
			total_amount: 0,
		}
	}
}
