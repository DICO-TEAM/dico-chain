//! # Farm Extend Pallet
//!
//! ## Overview
//!
//! Farm extend pallet provides functionality for managing liquidity mint.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};
use core::convert::{TryFrom};
use frame_support::{
	traits::{Get, EnsureOrigin},
	pallet_prelude::*, ensure, transactional, PalletId,
	sp_runtime::traits::{Zero, One, AtLeast32Bit, CheckedAdd},
	dispatch::DispatchErrorWithPostInfo,
};
use frame_system::pallet_prelude::*;
use dico_primitives::{AssetId, Balance, Amount, BlockNumber, to_u256, to_balance};
use sp_runtime::{ArithmeticError, traits::{AccountIdConversion, SaturatedConversion}};
use sp_core::U256;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PoolExtendStatus {
	Pending,
	Cancelled,
	InProgress,
	Finished,
}

#[derive(Encode, Decode, Default, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ParticipantExtend {
	pub amount: Balance,
	pub reward_debt: Balance,
}

impl ParticipantExtend {
	pub fn new(amount: Balance, reward_debt: Balance) -> Self {
		Self { amount, reward_debt }
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PoolExtendInfo<AccountId> {
	pub currency_id: AssetId,
	pub currency_amount: Balance,
	pub owner: AccountId,
	pub start_block: BlockNumber,
	pub end_block: BlockNumber,
	pub reward_per_block: Balance,
	pub last_reward_block: BlockNumber,
	pub acc_reward_per_share: Balance,
	pub stake_currency_id: AssetId,
	pub total_stake_amount: Balance,
	pub status: PoolExtendStatus,
}

impl<AccountId> PoolExtendInfo<AccountId> {
	pub fn new(
		currency_id: AssetId,
		currency_amount: Balance,
		owner: AccountId,
		start_block: BlockNumber,
		end_block: BlockNumber,
		reward_per_block: Balance,
		last_reward_block: BlockNumber,
		stake_currency_id: AssetId,
		status: PoolExtendStatus,
	) -> Self {
		Self {
			currency_id,
			currency_amount,
			owner,
			start_block,
			end_block,
			reward_per_block,
			last_reward_block,
			acc_reward_per_share: Balance::zero(),
			stake_currency_id,
			total_stake_amount: Balance::zero(),
			status,
		}
	}
}


// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The mining pool id
		type PoolExtendId: Parameter
			+ Member
			+ Into<u32>
			+ AtLeast32Bit
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId=AssetId,
			Balance=Balance,
			Amount=Amount
		>;

		/// The origin that is allowed to set or update parameter.
		type FounderSetOrigin: EnsureOrigin<Self::Origin>;

		/// The mining pool's module id, keep all assets in pool.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		MustBeDifferentAsset,
		InvalidBlockConfigure,
		StartBlockOutDate,
		InvalidRewardPerBlock,
		/// Liquidity already exists.
		LiquidityIdCreated,
		/// The total issuance of liquid assets is zero.
		LiquidityIdZeroIssuance,
		/// Pool already exists.
		PoolExisted,
		/// The mining pool does not exist.
		PoolNotFind,
		/// The user does not exist in the mining pool.
		UserNotFindInPool,
		/// Invalid withdrawal amount.
		InsufficientWithdrawAmount,
		/// No pool id available.
		NoPoolExtendIdAvailable,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Liquidity was Withdrawn. [who, pool id, liquidity amount]
		LpExtendWithdrawn(T::AccountId, T::PoolExtendId, Balance),
		/// Liquidity was deposited. [who, pool id, liquidity amount]
		LpExtendDeposited(T::AccountId, T::PoolExtendId, Balance),
		/// The mining pool was created. [pool id]
		PoolExtendCreated(T::AccountId, T::PoolExtendId, AssetId, Balance, AssetId),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_pool_extend_id)]
	pub type NextPoolExtendId<T: Config> = StorageValue<_, T::PoolExtendId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_pool_extend)]
	pub type PoolExtends<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::PoolExtendId,
		PoolExtendInfo<T::AccountId>,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_participant)]
	pub type ParticipantExtends<T: Config> = StorageDoubleMap<
		_, Twox64Concat,
		T::PoolExtendId,
		Twox64Concat,
		T::AccountId,
		ParticipantExtend,
		OptionQuery
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_id: AssetId,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			reward_per_block: Balance,
			stake_currency_id: AssetId
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(currency_id != stake_currency_id, Error::<T>::MustBeDifferentAsset);
			ensure!(reward_per_block > Balance::zero(), Error::<T>::InvalidRewardPerBlock);
			ensure!(end_block > start_block, Error::<T>::InvalidBlockConfigure);

			let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();
			let start_block: BlockNumber = start_block.saturated_into();
			let end_block: BlockNumber = end_block.saturated_into();
			ensure!(block_number <= start_block, Error::<T>::StartBlockOutDate);

			let block_delta = end_block
				.checked_sub(start_block).ok_or(ArithmeticError::Overflow)?
				.checked_add(BlockNumber::one()).ok_or(ArithmeticError::Overflow)?;
			let currency_amount = to_balance!(
				to_u256!(block_delta).checked_mul(to_u256!(reward_per_block)).ok_or(ArithmeticError::Overflow)?
			).map_err(|_| ArithmeticError::Overflow)?;

			let module_account_id = Self::account_id();
			T::Currency::transfer(currency_id, &who, &module_account_id, currency_amount)?;

			let pool_extend_status = if block_number == start_block { PoolExtendStatus::InProgress
			} else { PoolExtendStatus::Pending };

			let pool_extend_info = PoolExtendInfo::new(
				currency_id,
				currency_amount,
				who.clone(),
				start_block,
				end_block,
				reward_per_block,
				start_block,
				stake_currency_id,
				pool_extend_status,
			);

			let pool_extend_id = Self::get_next_pool_extend_id()?;
			PoolExtends::<T>::insert(pool_extend_id, pool_extend_info);

			Self::deposit_event(Event::PoolExtendCreated(
				who,
				pool_extend_id,
				currency_id,
				currency_amount,
				stake_currency_id
			));

			Ok(().into())
		}

	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn get_next_pool_extend_id() -> sp_std::result::Result<T::PoolExtendId, DispatchErrorWithPostInfo> {
		let next_pool_extend_id = Self::next_pool_extend_id();
		let new_pool_extend_id = next_pool_extend_id.checked_add(&One::one()).ok_or(Error::<T>::NoPoolExtendIdAvailable)?;
		NextPoolExtendId::<T>::put(new_pool_extend_id);

		Ok(next_pool_extend_id)
	}
}

