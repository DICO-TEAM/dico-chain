//! # Farm Extend Pallet
//!
//! ## Overview
//!
//! Farm extend pallet provides functionality for managing liquidity mint.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(deprecated)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};
use core::convert::TryFrom;
use dico_primitives::{to_balance, to_u256, Amount, AssetId, Balance, BlockNumber};
use frame_support::{
	dispatch::DispatchErrorWithPostInfo,
	ensure,
	pallet_prelude::*,
	sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One, Zero},
	traits::{Get},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, SaturatedConversion},
	ArithmeticError,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

mod benchmarking;
pub mod weights;
use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
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

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
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
		}
	}
}

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The mining pool id
		type PoolExtendId: Parameter + Member + Into<u32> + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>;

		/// The mining pool's module id, keep all assets in pool.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		MustBeDifferentAsset,
		InvalidBlockConfigure,
		StartBlockOutDate,
		InvalidRewardPerBlock,
		/// Pool already exists.
		PoolExtendExisted,
		/// The mining pool does not exist.
		PoolExtendNotFind,
		/// The user does not exist in the mining pool.
		UserNotFindInPoolExtend,
		/// Invalid withdrawal amount.
		InsufficientWithdrawAmount,
		/// No pool id available.
		NoPoolExtendIdAvailable,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Liquidity was Withdrawn. [who, pool id, liquidity amount]
		AssetWithdrawn(T::AccountId, T::PoolExtendId, Balance),
		/// Liquidity was deposited. [who, pool id, liquidity amount]
		AssetDeposited(T::AccountId, T::PoolExtendId, Balance),
		/// The mining pool was created. [pool id]
		PoolExtendCreated(T::AccountId, T::PoolExtendId, AssetId, Balance, AssetId),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_pool_extend_id)]
	pub type NextPoolExtendId<T: Config> = StorageValue<_, T::PoolExtendId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_pool_extend)]
	pub type PoolExtends<T: Config> =
		StorageMap<_, Twox64Concat, T::PoolExtendId, PoolExtendInfo<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_participant)]
	pub type ParticipantExtends<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::PoolExtendId, Twox64Concat, T::AccountId, ParticipantExtend, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(< T as Config >::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_id: AssetId,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			reward_per_block: Balance,
			stake_currency_id: AssetId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(currency_id != stake_currency_id, Error::<T>::MustBeDifferentAsset);
			ensure!(reward_per_block > Balance::zero(), Error::<T>::InvalidRewardPerBlock);
			ensure!(end_block > start_block, Error::<T>::InvalidBlockConfigure);

			let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();
			let start_block: BlockNumber = start_block.saturated_into();
			let end_block: BlockNumber = end_block.saturated_into();
			ensure!(block_number <= start_block, Error::<T>::StartBlockOutDate);

			let block_delta = end_block.checked_sub(start_block).ok_or(ArithmeticError::Overflow)?;
			let currency_amount = to_balance!(to_u256!(block_delta)
				.checked_mul(to_u256!(reward_per_block))
				.ok_or(ArithmeticError::Overflow)?)?;

			let module_account_id = Self::account_id();
			T::Currency::transfer(currency_id, &who, &module_account_id, currency_amount)?;

			let pool_extend_info = PoolExtendInfo::new(
				currency_id,
				currency_amount,
				who.clone(),
				start_block,
				end_block,
				reward_per_block,
				start_block,
				stake_currency_id,
			);

			let pool_extend_id = Self::get_next_pool_extend_id()?;
			PoolExtends::<T>::insert(pool_extend_id, pool_extend_info);

			Self::deposit_event(Event::PoolExtendCreated(
				who,
				pool_extend_id,
				currency_id,
				currency_amount,
				stake_currency_id,
			));

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::deposit_asset())]
		#[transactional]
		pub fn deposit_asset(
			origin: OriginFor<T>,
			pool_extend_id: T::PoolExtendId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::update_pool_extend(&pool_extend_id)?;
			let mut pool_extend = PoolExtends::<T>::get(pool_extend_id).ok_or(Error::<T>::PoolExtendNotFind)?;
			let mut participant_extend = ParticipantExtends::<T>::get(pool_extend_id, &who).unwrap_or_default();

			let module_account_id = Self::account_id();
			if participant_extend.amount > Balance::zero() {
				let pending_reward = to_balance!(to_u256!(participant_extend.amount)
					.checked_mul(to_u256!(pool_extend.acc_reward_per_share))
					.ok_or(ArithmeticError::Overflow)?
					.checked_div(to_u256!(1e12 as u64))
					.ok_or(ArithmeticError::Overflow)?
					.checked_sub(to_u256!(participant_extend.reward_debt))
					.ok_or(ArithmeticError::Overflow)?)?;

				if pending_reward > Balance::zero() {
					T::Currency::transfer(pool_extend.currency_id, &module_account_id, &who, pending_reward)?;
				}
			}

			if amount > Balance::zero() {
				T::Currency::transfer(pool_extend.stake_currency_id, &who, &module_account_id, amount)?;
				participant_extend.amount = participant_extend
					.amount
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?;
				pool_extend.total_stake_amount = pool_extend
					.total_stake_amount
					.checked_add(amount)
					.ok_or(ArithmeticError::Overflow)?;
			}

			participant_extend.reward_debt = to_balance!(to_u256!(participant_extend.amount)
				.checked_mul(to_u256!(pool_extend.acc_reward_per_share))
				.ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64))
				.ok_or(ArithmeticError::Overflow)?)?;

			PoolExtends::<T>::insert(pool_extend_id, pool_extend);
			ParticipantExtends::<T>::insert(pool_extend_id, &who, participant_extend);
			Self::deposit_event(Event::AssetDeposited(who, pool_extend_id, amount));

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::withdraw_asset())]
		#[transactional]
		pub fn withdraw_asset(
			origin: OriginFor<T>,
			pool_extend_id: T::PoolExtendId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut participant_extend =
				ParticipantExtends::<T>::get(pool_extend_id, &who).ok_or(Error::<T>::UserNotFindInPoolExtend)?;
			ensure!(
				participant_extend.amount >= amount,
				Error::<T>::InsufficientWithdrawAmount
			);

			Self::update_pool_extend(&pool_extend_id)?;

			let mut pool_extend = PoolExtends::<T>::get(pool_extend_id).ok_or(Error::<T>::PoolExtendNotFind)?;

			let pending_reward = to_balance!(to_u256!(participant_extend.amount)
				.checked_mul(to_u256!(pool_extend.acc_reward_per_share))
				.ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64))
				.ok_or(ArithmeticError::Overflow)?
				.checked_sub(to_u256!(participant_extend.reward_debt))
				.ok_or(ArithmeticError::Overflow)?)?;

			let module_account_id = Self::account_id();

			if pending_reward > Balance::zero() {
				T::Currency::transfer(pool_extend.currency_id, &module_account_id, &who, pending_reward)?;
			}

			if amount > Balance::zero() {
				participant_extend.amount = to_balance!(to_u256!(participant_extend.amount)
					.checked_sub(to_u256!(amount))
					.ok_or(ArithmeticError::Overflow)?)?;
				pool_extend.total_stake_amount = to_balance!(to_u256!(pool_extend.total_stake_amount)
					.checked_sub(to_u256!(amount))
					.ok_or(ArithmeticError::Overflow)?)?;
				T::Currency::transfer(pool_extend.stake_currency_id, &module_account_id, &who, amount)?;
			}

			participant_extend.reward_debt = to_balance!(to_u256!(participant_extend.amount)
				.checked_mul(to_u256!(pool_extend.acc_reward_per_share))
				.ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64))
				.ok_or(ArithmeticError::Overflow)?)?;

			PoolExtends::<T>::insert(pool_extend_id, pool_extend);
			ParticipantExtends::<T>::insert(pool_extend_id, &who, participant_extend);
			Self::deposit_event(Event::AssetWithdrawn(who, pool_extend_id, amount));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	fn get_next_pool_extend_id() -> sp_std::result::Result<T::PoolExtendId, DispatchErrorWithPostInfo> {
		let next_pool_extend_id = Self::next_pool_extend_id();
		let new_pool_extend_id = next_pool_extend_id
			.checked_add(&One::one())
			.ok_or(Error::<T>::NoPoolExtendIdAvailable)?;
		NextPoolExtendId::<T>::put(new_pool_extend_id);

		Ok(next_pool_extend_id)
	}

	fn update_pool_extend(pid: &T::PoolExtendId) -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
		PoolExtends::<T>::try_mutate(
			pid,
			|maybe_pool_extend| -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
				let pool_extend = maybe_pool_extend.as_mut().ok_or(Error::<T>::PoolExtendNotFind)?;
				let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();
				if block_number <= pool_extend.last_reward_block {
					return Ok(());
				}

				let reward_block = if block_number < pool_extend.end_block {
					block_number
				} else {
					pool_extend.end_block
				};
				let block_delta = reward_block
					.checked_sub(pool_extend.last_reward_block)
					.ok_or(ArithmeticError::Overflow)?;
				if block_delta == BlockNumber::zero() {
					return Ok(());
				}

				if pool_extend.total_stake_amount == Balance::zero() {
					pool_extend.last_reward_block = reward_block;
					return Ok(());
				}

				let block_rewards = to_balance!(to_u256!(block_delta)
					.checked_mul(to_u256!(pool_extend.reward_per_block))
					.ok_or(ArithmeticError::Overflow)?)?;

				pool_extend.acc_reward_per_share = to_balance!(to_u256!(pool_extend.acc_reward_per_share)
					.checked_add(
						to_u256!(block_rewards)
							.checked_mul(to_u256!(1e12 as u64))
							.ok_or(ArithmeticError::Overflow)?
							.checked_div(to_u256!(pool_extend.total_stake_amount))
							.ok_or(ArithmeticError::DivisionByZero)?
					)
					.ok_or(ArithmeticError::Overflow)?)?;
				pool_extend.last_reward_block = reward_block;

				Ok(())
			},
		)
	}
}
