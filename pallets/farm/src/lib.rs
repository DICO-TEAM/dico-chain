//! # Farm Pallet
//!
//! ## Overview
//!
//! Farm pallet provides functionality for managing liquidity mint.

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
use sp_std::{vec, vec::Vec};
use frame_system::pallet_prelude::*;
use dico_primitives::{AssetId, Balance, Amount, BlockNumber, to_u256, to_balance};
use sp_runtime::{ArithmeticError, traits::{AccountIdConversion, SaturatedConversion}};
use sp_core::U256;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

mod benchmarking;

pub mod weights;

use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Participant {
	pub amount: Balance,
	pub reward_debt: Balance,
}

impl Participant {
	pub fn new(amount: Balance, reward_debt: Balance) -> Self {
		Self { amount, reward_debt }
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Default, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PoolInfo {
	pub currency_id: AssetId,
	pub alloc_point: u128,
	pub last_reward_block: BlockNumber,
	pub acc_dico_per_share: Balance,
	pub total_amount: Balance,
	pub start_block: BlockNumber,
	pub end_block: BlockNumber,
}

impl PoolInfo {
	pub fn new(
		currency_id: AssetId,
		alloc_point: u128,
		last_reward_block: BlockNumber,
		start_block: BlockNumber,
		end_block: BlockNumber,
	) -> Self {
		Self {
			currency_id,
			alloc_point,
			last_reward_block,
			acc_dico_per_share: Balance::zero(),
			total_amount: Balance::zero(),
			start_block,
			end_block,
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
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			let _ = Self::update_pool_alloc_point_gradually(now);
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The mining pool id
		type PoolId:
		Parameter
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

		/// Native Asset Id
		#[pallet::constant]
		type NativeAssetId: Get<AssetId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Block configured is invalid.
		InvalidBlockConfigured,
		/// Liquidity already exists.
		LiquidityIdCreated,
		/// The total issuance of liquid assets is zero.
		LiquidityIdZeroIssuance,
		/// The total allocation point is zero.
		TotalAllocPointIsZero,
		/// Pool already exists.
		PoolExisted,
		/// The mining pool does not exist.
		PoolNotFind,
		/// The user does not exist in the mining pool.
		UserNotFindInPool,
		/// Invalid withdrawal amount.
		InsufficientWithdrawAmount,
		/// No pool id available.
		NoPoolIdAvailable,
		/// Pool liquidity balance is zero.
		PoolLiquidityBalanceIsZero,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The mining reward halving cycle was set. [block number]
		HalvingPeriodIsSet(T::BlockNumber),
		/// The reward for each block was set. [dico amount]
		DicoPerBlockIsSet(Balance),
		/// The block number of the mining pool to start mining was set. [block number]
		StartBlockIsSet(T::BlockNumber),
		/// Liquidity was Withdrawn. [who, pool id, liquidity amount]
		LpWithdrawn(T::AccountId, T::PoolId, Balance),
		/// Liquidity was deposited. [who, pool id, liquidity amount]
		LpDeposited(T::AccountId, T::PoolId, Balance),
		/// The mining pool was created. [pool id]
		PoolCreated(T::PoolId),
		/// The allocated points of each designated mining pool was updated. [pool id, alloc point]
		PoolAllocPointUpdated(T::PoolId, u128),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_pools)]
	pub type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolInfo, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_users)]
	pub type Participants<T: Config> = StorageDoubleMap<
		_, Twox64Concat,
		T::PoolId,
		Twox64Concat,
		T::AccountId,
		Participant,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn dico_per_block)]
	pub type DicoPerBlock<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_alloc_point)]
	pub type TotalAllocPoint<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn start_block)]
	pub type StartBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn halving_period)]
	pub type HalvingPeriod<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the mining reward halving cycle,the unit is the number of blocks.
		///
		/// This method needs to be called before the mining pool is created.
		/// Once mining starts, do not call this method again, otherwise it may
		///  cause an error in the reward calculation.
		/// Emits `HalvingPeriodIsSet` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::set_halving_period())]
		#[transactional]
		pub fn set_halving_period(
			origin: OriginFor<T>,
			block_number: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;
			HalvingPeriod::<T>::set(block_number);
			Self::deposit_event(Event::HalvingPeriodIsSet(block_number));

			Ok(().into())
		}

		/// Set the reward for each block when starting mining.
		///
		/// The actual mining reward for each block depends on this parameter and the halving cycle.
		///
		/// This method needs to be called before the mining pool is created.
		/// Once mining starts, do not call this method again, otherwise it may
		///  cause an error in the reward calculation.
		///
		/// Emits `DicoPerBlockIsSet` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::set_dico_per_block())]
		#[transactional]
		pub fn set_dico_per_block(
			origin: OriginFor<T>,
			new_per_block: Balance,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;
			DicoPerBlock::<T>::set(new_per_block);
			Self::deposit_event(Event::DicoPerBlockIsSet(new_per_block));

			Ok(().into())
		}

		/// Set the block number of the mining pool to start mining.
		///
		/// This method needs to be called before the mining pool is created.
		/// Once mining starts, do not call this method again, otherwise it may
		/// cause an error in the reward calculation.
		///
		/// Emits `StartBlockIsSet` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::set_start_block())]
		#[transactional]
		pub fn set_start_block(
			origin: OriginFor<T>,
			block_number: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;
			StartBlock::<T>::set(block_number);
			Self::deposit_event(Event::StartBlockIsSet(block_number));

			Ok(().into())
		}

		/// Update the allocated points of each designated mining pool.
		///
		/// This method can be called multiple times without causing a reward calculation error.
		///
		/// Emits `PoolAllocPointUpdated` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::update_pool_alloc_point())]
		#[transactional]
		pub fn update_pool_alloc_point(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			alloc_point: u128,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;

			Self::mass_update_pools()?;

			let mut pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFind)?;

			let mut total_alloc_point = TotalAllocPoint::<T>::get();
			total_alloc_point = total_alloc_point
				.checked_sub(pool.alloc_point).ok_or(ArithmeticError::Overflow)?
				.checked_add(alloc_point).ok_or(ArithmeticError::Overflow)?;
			TotalAllocPoint::<T>::put(total_alloc_point);

			pool.alloc_point = alloc_point;
			Pools::<T>::insert(pool_id, pool);

			Self::deposit_event(Event::PoolAllocPointUpdated(pool_id, alloc_point));

			Ok(().into())
		}

		/// Create a new mining pool.
		///
		/// The liquidity id can be the lp id or the asset id of a single currency.
		///
		/// Emits `PoolCreated` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_id: AssetId,
			alloc_point: u128,
			start_block: BlockNumber,
			end_block: BlockNumber,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;

			let farm_start_block: BlockNumber = Self::start_block().saturated_into();
			ensure!(start_block < end_block && start_block >= farm_start_block,
				Error::<T>::InvalidBlockConfigured);

			let total_issuance = T::Currency::total_issuance(currency_id);
			ensure!(total_issuance != 0, Error::<T>::LiquidityIdZeroIssuance);

			for (_, pool_info) in Pools::<T>::iter() {
				ensure!(pool_info.currency_id != currency_id, Error::<T>::LiquidityIdCreated);
			}

			let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();

			let next_pool_id = Self::get_next_pool_id()?;

			Self::mass_update_pools()?;

			let block_arr = vec![block_number, start_block, farm_start_block];
			let last_reward_block = *block_arr.iter().max().ok_or(Error::<T>::InvalidBlockConfigured)?;

			let mut total_alloc_point = TotalAllocPoint::<T>::get();
			total_alloc_point = total_alloc_point
				.checked_add(alloc_point).ok_or(ArithmeticError::Overflow)?;
			TotalAllocPoint::<T>::put(total_alloc_point);

			let pool_info = PoolInfo::new(
				currency_id,
				alloc_point,
				last_reward_block,
				start_block,
				end_block,
			);
			Pools::<T>::insert(next_pool_id, pool_info);
			Self::deposit_event(Event::PoolCreated(next_pool_id));

			Ok(().into())
		}

		/// Deposit liquid assets to designated mining pools to participate in mining.
		///
		/// Emits `LpDeposited` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::deposit_lp())]
		#[transactional]
		pub fn deposit_lp(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::update_pool(&pool_id)?;

			let mut pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFind)?;
			let mut participant = Participants::<T>::get(pool_id, &who).unwrap_or_default();

			let (user_amount_hp, user_reward_debt_hp) = to_u256!(participant.amount, participant.reward_debt);
			let (acc_dico_per_share_hp, total_amount_hp) = to_u256!(pool.acc_dico_per_share, pool.total_amount);

			let module_account_id = Self::account_id();
			let native_asset = T::NativeAssetId::get();
			let amount_hp = to_u256!(amount);

			if participant.amount > Balance::zero() {
				let pending_amount = user_amount_hp
					.checked_mul(acc_dico_per_share_hp).ok_or(ArithmeticError::Overflow)?
					.checked_div(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?
					.checked_sub(user_reward_debt_hp).ok_or(ArithmeticError::Overflow)?;
				if pending_amount > U256::zero() {
					T::Currency::transfer(native_asset, &module_account_id, &who, to_balance!(pending_amount)?)?;
				}
			}

			if amount > Balance::zero() {
				T::Currency::transfer(pool.currency_id, &who, &module_account_id, amount)?;
				let total_amount = user_amount_hp
					.checked_add(amount_hp).ok_or(ArithmeticError::Overflow)?;
				participant.amount = to_balance!(total_amount)?;
				pool.total_amount = to_balance!(total_amount_hp
					.checked_add(amount_hp).ok_or(ArithmeticError::Overflow)?)?;
			}

			participant.reward_debt = to_balance!(to_u256!(participant.amount)
				.checked_mul(acc_dico_per_share_hp).ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?)?;

			Pools::<T>::insert(pool_id, pool);
			Participants::<T>::insert(pool_id, &who, participant);
			Self::deposit_event(Event::LpDeposited(who, pool_id, amount));

			Ok(().into())
		}

		/// Withdraw liquidity.
		///
		/// if amount = 0, then only withdraw mining harvest,and liquidity assets remain unchanged.
		/// if amount > 0, in addition to the withdrawal of mining rewards, the amount of assets
		/// will also be withdrawn.
		///
		/// Emits `LpWithdrawn` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::withdraw_lp())]
		#[transactional]
		pub fn withdraw_lp(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut participant = Participants::<T>::get(pool_id, &who).ok_or(Error::<T>::UserNotFindInPool)?;
			ensure!(participant.amount >= amount, Error::<T>::InsufficientWithdrawAmount);

			Self::update_pool(&pool_id)?;

			let mut pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFind)?;

			let pending_amount = to_u256!(participant.amount)
				.checked_mul(to_u256!(pool.acc_dico_per_share)).ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?
				.checked_sub(to_u256!(participant.reward_debt)).ok_or(ArithmeticError::Overflow)?;

			let module_account_id = Self::account_id();
			let native_asset = T::NativeAssetId::get();

			if pending_amount > U256::zero() {
				T::Currency::transfer(native_asset, &module_account_id, &who, to_balance!(pending_amount)?)?;
			}

			if amount > Balance::zero() {
				participant.amount = to_balance!(to_u256!(participant.amount)
					.checked_sub(to_u256!(amount)).ok_or(ArithmeticError::Overflow)?)?;
				pool.total_amount = to_balance!(to_u256!(pool.total_amount)
					.checked_sub(to_u256!(amount)).ok_or(ArithmeticError::Overflow)?)?;
				T::Currency::transfer(pool.currency_id, &module_account_id, &who, amount)?;
			}

			participant.reward_debt = to_balance!(to_u256!(participant.amount)
				.checked_mul(to_u256!(pool.acc_dico_per_share)).ok_or(ArithmeticError::Overflow)?
				.checked_div(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?)?;

			Pools::<T>::insert(pool_id, pool);
			Participants::<T>::insert(pool_id, &who, participant);
			Self::deposit_event(Event::LpWithdrawn(who, pool_id, amount));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn get_next_pool_id() -> sp_std::result::Result<T::PoolId, DispatchErrorWithPostInfo> {
		let next_pool_id = Self::next_pool_id();
		let new_pool_id = next_pool_id.checked_add(&One::one()).ok_or(Error::<T>::NoPoolIdAvailable)?;
		NextPoolId::<T>::put(new_pool_id);

		Ok(next_pool_id)
	}

	/// Calculate the number of halving of the specified block,
	/// the return value is 0,1,...N, which means the Nth halving
	fn phase(block_number: BlockNumber) -> sp_std::result::Result<U256, ArithmeticError> {
		let halving_period: BlockNumber = Self::halving_period().saturated_into();
		let start_block: BlockNumber = Self::start_block().saturated_into();

		let (halving_period_hp, start_block_hp, block_number_hp)
			= to_u256!(halving_period, start_block, block_number);

		if halving_period.is_zero() {
			return Ok(U256::zero());
		}

		if block_number > start_block {
			let halving_number = block_number_hp
				.checked_sub(start_block_hp).ok_or(ArithmeticError::Overflow)?
				.checked_sub(U256::one()).ok_or(ArithmeticError::Overflow)?
				.checked_div(halving_period_hp).ok_or(ArithmeticError::Overflow)?;

			return Ok(halving_number);
		}

		Ok(U256::zero())
	}

	/// Calculate the number of DICOs produced by the specified block. This is easy to calculate.
	/// Divide the DicoPerBlock set at the initial stage by the Nth power of 2,
	/// where N represents the number of halvings.
	fn reward(block_number: BlockNumber) -> sp_std::result::Result<Balance, ArithmeticError> {
		let phase = Self::phase(block_number)?;

		let dico_per_block = to_u256!(Self::dico_per_block());
		let rewards = dico_per_block
			.checked_div(U256::from(2)
				.checked_pow(phase).ok_or(ArithmeticError::Overflow)?)
			.ok_or(ArithmeticError::Overflow)?;

		to_balance!(rewards)
	}

	/// Calculate the number of DICOs produced from the last reward block to the specific block cycle.
	fn get_dico_block_reward(
		last_reward_block: BlockNumber,
		specific_block: BlockNumber
	) -> sp_std::result::Result<Balance, ArithmeticError> {
		let mut block_reward = U256::zero();
		let halving_period: BlockNumber = Self::halving_period().saturated_into();
		let start_block: BlockNumber = Self::start_block().saturated_into();

		if specific_block <= last_reward_block {
			return to_balance!(0);
		}

		let mut n = Self::phase(last_reward_block)?;
		let m = Self::phase(specific_block)?;

		let mut last_reward_block = to_u256!(last_reward_block);

		while n < m {
			n = n.checked_add(U256::one()).ok_or(ArithmeticError::Overflow)?;
			let r = n
				.checked_mul(to_u256!(halving_period)).ok_or(ArithmeticError::Overflow)?
				.checked_add(to_u256!(start_block)).ok_or(ArithmeticError::Overflow)?;

			ensure!(r <= to_u256!(BlockNumber::MAX), ArithmeticError::Overflow);

			let r_reward = Self::reward(BlockNumber::from(r.as_u32()))?;
			let r_block_reward = r
				.checked_sub(last_reward_block).ok_or(ArithmeticError::Overflow)?
				.checked_mul(to_u256!(r_reward)).ok_or(ArithmeticError::Overflow)?;

			block_reward = block_reward
				.checked_add(r_block_reward).ok_or(ArithmeticError::Overflow)?;
			last_reward_block = r;
		}

		let y_block_reward = to_u256!(specific_block)
			.checked_sub(to_u256!(last_reward_block)).ok_or(ArithmeticError::Overflow)?
			.checked_mul(to_u256!(Self::reward(specific_block)?)).ok_or(ArithmeticError::Overflow)?;

		block_reward = block_reward.checked_add(y_block_reward).ok_or(ArithmeticError::Overflow)?;

		to_balance!(block_reward)
	}

	fn calc_participant_reward(
		account: T::AccountId,
		pid: T::PoolId,
	) -> sp_std::result::Result<Balance, DispatchErrorWithPostInfo> {
		let pool = Pools::<T>::get(pid).ok_or(Error::<T>::PoolNotFind)?;
		let participant = Participants::<T>::get(pid, account).ok_or(Error::<T>::UserNotFindInPool)?;

		let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();

		if block_number <= pool.last_reward_block
			|| block_number < pool.start_block {
			return Ok(Balance::zero());
		}

		let reward_block = if block_number > pool.last_reward_block && block_number >= pool.end_block {
			pool.end_block
		} else { block_number };

		let block_reward = Self::get_dico_block_reward(pool.last_reward_block, reward_block)?;
		let dico_reward = to_u256!(block_reward)
			.checked_mul(to_u256!(pool.alloc_point)).ok_or(ArithmeticError::Overflow)?
			.checked_div(to_u256!(Self::total_alloc_point())).ok_or(ArithmeticError::Overflow)?;

		let module_account_id = Self::account_id();
		let lp_supply = T::Currency::free_balance(pool.currency_id, &module_account_id);
		ensure!(!lp_supply.is_zero(), Error::<T>::PoolLiquidityBalanceIsZero);

		let acc_dico_per_share = to_u256!(pool.acc_dico_per_share)
			.checked_add(
				dico_reward
					.checked_mul(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?
					.checked_div(to_u256!(lp_supply)).ok_or(ArithmeticError::Overflow)?
			).ok_or(ArithmeticError::Overflow)?;

		let pending_reward = to_u256!(participant.amount)
			.checked_mul(to_u256!(acc_dico_per_share)).ok_or(ArithmeticError::Overflow)?
			.checked_div(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?
			.checked_sub(to_u256!(participant.reward_debt)).ok_or(ArithmeticError::Overflow)?;

		let pending_reward = to_balance!(pending_reward)?;
		Ok(pending_reward)
	}

	pub fn get_participant_reward(account: T::AccountId, pid: T::PoolId) -> Balance {
		if let Ok(pending_reward) = Self::calc_participant_reward(account, pid) {
			return pending_reward;
		}

		Balance::zero()
	}

	pub fn update_pool_alloc_point_gradually(
		block_number: T::BlockNumber
	) -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
		let block_number: BlockNumber = block_number.saturated_into();
		let pools = Pools::<T>::iter().collect::<Vec<_>>();
		let mut mass_update_pool = false;
		for (pid, mut pool) in pools {
			if block_number >= pool.end_block && pool.alloc_point > 0u128 {
				if !mass_update_pool {
					Self::mass_update_pools()?;
					mass_update_pool = true;
				}

				let mut total_alloc_point = TotalAllocPoint::<T>::get();
				total_alloc_point = total_alloc_point
					.checked_sub(pool.alloc_point).ok_or(ArithmeticError::Overflow)?;
				TotalAllocPoint::<T>::put(total_alloc_point);

				pool.alloc_point = 0u128;
				Pools::<T>::insert(pid, pool);
			}
		}

		Ok(())
	}

	fn update_pool(pid: &T::PoolId) -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
		Pools::<T>::try_mutate(pid, |maybe_pool| -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
			let pool = maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFind)?;

			let block_number: BlockNumber = frame_system::pallet::Pallet::<T>::block_number().saturated_into();

			if block_number <= pool.last_reward_block
				|| block_number < pool.start_block
				|| pool.last_reward_block >= pool.end_block {
				return Ok(());
			}

			let reward_block = if block_number > pool.last_reward_block && block_number >= pool.end_block {
				pool.end_block
			} else { block_number };

			let module_account_id = Self::account_id();
			let native_asset = T::NativeAssetId::get();
			let lp_supply = T::Currency::free_balance(pool.currency_id, &module_account_id);
			if lp_supply.is_zero() {
				pool.last_reward_block = reward_block;
				return Ok(());
			}

			// Calculate the number of DICOs produced from the last reward block to the current block cycle.
			let block_reward = Self::get_dico_block_reward(pool.last_reward_block, reward_block)?;
			if block_reward.is_zero() {
				return Ok(());
			}

			// According to the mining weight value of the mining pool,
			// the number of DICOs that the mining pool can allocate in this period is calculated.
			let total_alloc_point = to_u256!(Self::total_alloc_point());

			ensure!(total_alloc_point != U256::zero(), Error::<T>::TotalAllocPointIsZero);

			let dico_reward = to_u256!(block_reward)
				.checked_mul(to_u256!(pool.alloc_point)).ok_or(ArithmeticError::Overflow)?
				.checked_div(total_alloc_point).ok_or(ArithmeticError::Overflow)?;

			// Call the minting interface to mint DICO for the module.
			T::Currency::deposit(native_asset, &module_account_id, to_balance!(dico_reward)?)?;

			pool.acc_dico_per_share = to_balance!(to_u256!(pool.acc_dico_per_share)
				.checked_add(
					dico_reward
						.checked_mul(to_u256!(1e12 as u64)).ok_or(ArithmeticError::Overflow)?
						.checked_div( to_u256!(lp_supply)).ok_or(ArithmeticError::Overflow)?
				).ok_or(ArithmeticError::Overflow)?)?;

			pool.last_reward_block = reward_block;

			Ok(())
		})
	}

	/// Update the reward variables of all pools. Be careful of gas consumption!
	fn mass_update_pools() -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
		for (pid, _) in Pools::<T>::iter() {
			Self::update_pool(&pid)?;
		}

		Ok(())
	}
}
