//! # Lbp Pallet
//!
//! ## Overview
//!
//! Lbp pallet provides functionality for managing liquidity bootstrapping pool and executing trades.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]


use core::convert::TryFrom;
use frame_support::{
	// log,
	traits::Get,
	pallet_prelude::*, ensure, transactional, PalletId,
	sp_runtime::traits::{Zero, One, AtLeast32Bit, CheckedAdd},
	dispatch::DispatchErrorWithPostInfo,
};
use frame_system::pallet_prelude::*;
use dico_primitives::{AssetId, Balance, Amount, BlockNumber};
use sp_std::{vec::Vec};
use sp_runtime::{ArithmeticError, traits::{AccountIdConversion, SaturatedConversion}, RuntimeDebug};
use sp_core::U256;
use dico_primitives::{to_u256, to_balance};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

mod benchmarking;

pub mod weights;

use weights::WeightInfo;

mod math;

use math::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};


#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LbpStatus {
	Pending,
	Cancelled,
	InProgress,
	Finished,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LbpInfo<AccountId> {
	pub start_block: BlockNumber,
	pub end_block: BlockNumber,
	pub steps: u32,
	pub owner: AccountId,
	// Investors pay for asset
	pub supply_asset: AssetId,
	// Asset purchased by investors
	pub target_asset: AssetId,
	pub initial_supply_balance: Balance,
	pub initial_target_balance: Balance,
	pub initial_supply_start_weight: u128,
	pub initial_supply_end_weight: u128,
	pub initial_target_start_weight: u128,
	pub initial_target_end_weight: u128,

	pub supply_balance: Balance,
	pub target_balance: Balance,
	pub supply_weight: u128,
	pub target_weight: u128,
	pub status: LbpStatus,
	pub step: u32,
	pub next_block: BlockNumber,
}

impl<AccountId> LbpInfo<AccountId> {
	pub fn new(
		start_block: BlockNumber,
		end_block: BlockNumber,
		steps: u32,
		owner: AccountId,
		supply_asset: AssetId,
		target_asset: AssetId,
		initial_supply_balance: u128,
		initial_target_balance: u128,
		initial_supply_start_weight: u128,
		initial_supply_end_weight: u128,
		initial_target_start_weight: u128,
		initial_target_end_weight: u128,
	) -> Self {
		Self {
			start_block,
			end_block,
			steps,
			owner,
			supply_asset,
			target_asset,
			initial_supply_balance,
			initial_target_balance,
			initial_supply_start_weight,
			initial_supply_end_weight,
			initial_target_start_weight,
			initial_target_end_weight,
			supply_balance: initial_supply_balance,
			target_balance: initial_target_balance,
			supply_weight: initial_supply_start_weight,
			target_weight: initial_target_start_weight,
			status: LbpStatus::Pending,
			step: 0u32,
			next_block: BlockNumber::zero(),
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LbpPair {
	pub supply_asset: AssetId,
	pub target_asset: AssetId,
}

impl LbpPair {
	pub fn new(supply_asset: AssetId, target_asset: AssetId) -> Self {
		Self { supply_asset, target_asset }
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
			Self::update_weights_gradually(now);
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId=AssetId, Balance=Balance, Amount=Amount>;

		/// The LBP's module id, keep all assets in LBP.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The mining pool id
		type LbpId: Parameter + Member + Into<u32> + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		NoLbpIdAvailable,
		LbpNotFind,
		OngoingLbpNotFind,
		LbpPairOngoing,
		MustBeOwner,
		MustBeNonTradingStatus,
		MustBeNotEndStatus,
		IncorrectSupplyAsset,
		IncorrectTargetAsset,
		MustBeInProgressStatus,
		InvalidSupplyAmount,
		InvalidTargetAmount,
		UnacceptableSupplyAmount,
		UnacceptableTargetAmount,
		ErrMathApprox,
		ErrMaxSupplyRatio,
		ErrMaxTargetRatio,
		ErrMinSteps,
		ErrMaxSteps,
		ErrMinWeight,
		ErrMaxWeight,
		ErrStartEndBlock,
		ErrMaxDurationBlock,
		ErrMinDurationBlock,
		UnacceptablePrice,
		StartBlockOutDate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposit test asset.
		AssetDeposited(T::AccountId, AssetId, Balance),
		LbpCreated(T::AccountId, T::LbpId, AssetId, AssetId, Balance, Balance),
		LbpExited(T::AccountId, T::LbpId),
		Swapped(T::AccountId, T::LbpId, AssetId, AssetId, Balance, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_lbp_id)]
	pub type NextLbpId<T: Config> = StorageValue<_, T::LbpId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_lbp)]
	pub type Lbps<T: Config> = StorageMap<
		_, Blake2_128Concat, T::LbpId, LbpInfo<T::AccountId>, OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_ongoing_lbp)]
	pub type OngoingLbps<T: Config> = StorageMap<
		_, Blake2_128Concat, LbpPair, (T::AccountId, T::LbpId), OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_history)]
	pub type PriceHistory<T: Config> = StorageMap<
		_, Blake2_128Concat, T::LbpId, Vec<(BlockNumber, Balance, Balance, u128, u128)>, ValueQuery
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(< T as Config >::WeightInfo::create_lbp())]
		#[transactional]
		pub fn create_lbp(
			origin: OriginFor<T>,
			supply_asset: AssetId,
			target_asset: AssetId,
			supply_balance: Balance,
			target_balance: Balance,
			supply_start_weight: u128,
			supply_end_weight: u128,
			target_start_weight: u128,
			target_end_weight: u128,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			steps: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(steps >= MIN_STEPS, Error::<T>::ErrMinSteps);
			ensure!(steps <= MAX_STEPS, Error::<T>::ErrMaxSteps);
			ensure!(start_block < end_block, Error::<T>::ErrStartEndBlock);

			let initial_start_block: BlockNumber = start_block.saturated_into();
			let initial_end_block: BlockNumber = end_block.saturated_into();
			let duration_block = initial_end_block
				.checked_sub(initial_start_block).ok_or(ArithmeticError::Overflow)?;
			ensure!(duration_block <= MAX_DURATION_BLOCK, Error::<T>::ErrMaxDurationBlock);
			ensure!(duration_block >= MIN_DURATION_BLOCK, Error::<T>::ErrMinDurationBlock);

			Self::ensure_weight(supply_start_weight)?;
			Self::ensure_weight(supply_end_weight)?;
			Self::ensure_weight(target_start_weight)?;
			Self::ensure_weight(target_end_weight)?;

			let lbp_pair = LbpPair::new(supply_asset, target_asset);
			ensure!(!OngoingLbps::<T>::contains_key(lbp_pair), Error::<T>::LbpPairOngoing);

			let current_block_number = frame_system::pallet::Pallet::<T>::block_number();
			let block_number: BlockNumber = current_block_number.saturated_into();

			ensure!(block_number <= initial_start_block, Error::<T>::StartBlockOutDate);

			let lbp_id = Self::get_next_lbp_id()?;

			let mut lbp_info = LbpInfo::new(
				initial_start_block,
				initial_end_block,
				steps,
				who.clone(),
				supply_asset,
				target_asset,
				supply_balance,
				target_balance,
				supply_start_weight,
				supply_end_weight,
				target_start_weight,
				target_end_weight,
			);

			if block_number == initial_start_block {
				lbp_info.status = LbpStatus::InProgress;
			}

			let module_account_id = Self::account_id();

			T::Currency::transfer(supply_asset, &who, &module_account_id, supply_balance)?;
			T::Currency::transfer(target_asset, &who, &module_account_id, target_balance)?;

			Lbps::<T>::insert(lbp_id, lbp_info);
			OngoingLbps::<T>::insert(lbp_pair, (who.clone(), lbp_id));

			Self::deposit_event(
				Event::LbpCreated(who, lbp_id, supply_asset,
								  target_asset, supply_balance, target_balance)
			);

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::exit_lbp())]
		#[transactional]
		pub fn exit_lbp(
			origin: OriginFor<T>,
			lbp_id: T::LbpId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut lbp_info = Lbps::<T>::get(lbp_id)
				.ok_or(Error::<T>::LbpNotFind)?;

			ensure!(lbp_info.owner == who, Error::<T>::MustBeOwner);
			ensure!(lbp_info.status == LbpStatus::Pending ||
                lbp_info.status == LbpStatus::Finished, Error::<T>::MustBeNonTradingStatus);

			let module_account_id = Self::account_id();

			T::Currency::transfer(lbp_info.supply_asset.clone(), &module_account_id,
								  &who, lbp_info.supply_balance)?;
			T::Currency::transfer(lbp_info.target_asset.clone(), &module_account_id,
								  &who, lbp_info.target_balance)?;

			lbp_info.supply_balance = Balance::zero();
			lbp_info.target_balance = Balance::zero();
			if lbp_info.status == LbpStatus::Pending {
				lbp_info.status = LbpStatus::Cancelled;
			}

			let lbp_pair = LbpPair::new(lbp_info.supply_asset.clone(),
										lbp_info.target_asset.clone());
			OngoingLbps::<T>::remove(lbp_pair);

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(Event::LbpExited(who, lbp_id));

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::swap_exact_amount_supply())]
		#[transactional]
		pub fn swap_exact_amount_supply(
			origin: OriginFor<T>,
			supply_asset: AssetId,
			supply_amount: Balance,
			target_asset: AssetId,
			min_target_amount: Balance,
			max_price: u128,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// let test_swap_fee = 1500000000000000u128;
			let test_swap_fee: u128 = 0;

			let lbp_pair = LbpPair::new(supply_asset, target_asset);
			let ongoing_lbp = OngoingLbps::<T>::try_get(lbp_pair).
				map_err(|_| Error::<T>::OngoingLbpNotFind)?;

			let lbp_id = ongoing_lbp.1;
			let mut lbp_info = Lbps::<T>::get(lbp_id)
				.ok_or(Error::<T>::LbpNotFind)?;

			ensure!(lbp_info.supply_asset == supply_asset, Error::<T>::IncorrectSupplyAsset);
			ensure!(lbp_info.target_asset == target_asset, Error::<T>::IncorrectTargetAsset);
			ensure!(lbp_info.status == LbpStatus::InProgress, Error::<T>::MustBeInProgressStatus);
			ensure!(min_target_amount <= lbp_info.target_balance, Error::<T>::InvalidTargetAmount);
			ensure!(
                to_u256!(supply_amount) <= bmul(
                    to_u256!(lbp_info.supply_balance),
                    to_u256!(MAX_IN_RATIO)
                ).ok_or(ArithmeticError::Overflow)?,
                Error::<T>::ErrMaxSupplyRatio
            );

			let spot_price_before =
				calc_spot_price(to_u256!(lbp_info.supply_balance),
								to_u256!(lbp_info.supply_weight),
								to_u256!(lbp_info.target_balance),
								to_u256!(lbp_info.target_weight),
								to_u256!(test_swap_fee),
				)?;

			let target_amount =
				calc_out_given_in(to_u256!(lbp_info.supply_balance),
								  to_u256!(lbp_info.supply_weight),
								  to_u256!(lbp_info.target_balance),
								  to_u256!(lbp_info.target_weight),
								  to_u256!(supply_amount),
								  to_u256!(test_swap_fee),
				)?;

			ensure!(target_amount >= to_u256!(min_target_amount), Error::<T>::UnacceptableTargetAmount);
			ensure!(
                spot_price_before <= bdiv(to_u256!(supply_amount), target_amount).ok_or(ArithmeticError::Overflow)?,
                Error::<T>::ErrMathApprox
            );

			lbp_info.supply_balance = to_balance!(
                badd(to_u256!(lbp_info.supply_balance), to_u256!(supply_amount)).ok_or(ArithmeticError::Overflow)?
            )?;

			lbp_info.target_balance = to_balance!(
                bsub(to_u256!(lbp_info.target_balance), target_amount).ok_or(ArithmeticError::Overflow)?
            )?;

			let spot_price_after =
				calc_spot_price(to_u256!(lbp_info.supply_balance),
								to_u256!(lbp_info.supply_weight),
								to_u256!(lbp_info.target_balance),
								to_u256!(lbp_info.target_weight),
								to_u256!(test_swap_fee),
				)?;

			ensure!(spot_price_after >= spot_price_before, Error::<T>::ErrMathApprox);
			ensure!(spot_price_after <= to_u256!(max_price), Error::<T>::UnacceptablePrice);

			let module_account_id = Self::account_id();

			T::Currency::transfer(supply_asset, &who,
								  &module_account_id, supply_amount)?;
			T::Currency::transfer(target_asset, &module_account_id,
								  &who, to_balance!(target_amount)?)?;

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(
				Event::Swapped(who, lbp_id, supply_asset,
							   target_asset, supply_amount, to_balance!(target_amount)?,
				)
			);

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::swap_exact_amount_target())]
		#[transactional]
		pub fn swap_exact_amount_target(
			origin: OriginFor<T>,
			supply_asset: AssetId,
			max_supply_amount: Balance,
			target_asset: AssetId,
			target_amount: Balance,
			max_price: u128,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let lbp_pair = LbpPair::new(supply_asset, target_asset);
			let ongoing_lbp = OngoingLbps::<T>::try_get(lbp_pair).
				map_err(|_| Error::<T>::OngoingLbpNotFind)?;

			let lbp_id = ongoing_lbp.1;
			let mut lbp_info = Lbps::<T>::get(lbp_id)
				.ok_or(Error::<T>::LbpNotFind)?;

			ensure!(lbp_info.supply_asset == supply_asset, Error::<T>::IncorrectSupplyAsset);
			ensure!(lbp_info.target_asset == target_asset, Error::<T>::IncorrectTargetAsset);
			ensure!(lbp_info.status == LbpStatus::InProgress, Error::<T>::MustBeInProgressStatus);
			ensure!(target_amount <= lbp_info.target_balance, Error::<T>::InvalidTargetAmount);
			ensure!(
                to_u256!(target_amount) <= bmul(
                    to_u256!(lbp_info.target_balance),
                    to_u256!(MAX_OUT_RATIO)
                ).ok_or(ArithmeticError::Overflow)?,
                Error::<T>::ErrMaxTargetRatio
            );

			let spot_price_before =
				calc_spot_price(to_u256!(lbp_info.supply_balance),
								to_u256!(lbp_info.supply_weight),
								to_u256!(lbp_info.target_balance),
								to_u256!(lbp_info.target_weight),
								to_u256!(0),
				)?;
			ensure!(spot_price_before <= to_u256!(max_price), Error::<T>::UnacceptablePrice);

			let supply_amount =
				calc_in_given_out(to_u256!(lbp_info.supply_balance),
								  to_u256!(lbp_info.supply_weight),
								  to_u256!(lbp_info.target_balance),
								  to_u256!(lbp_info.target_weight),
								  to_u256!(target_amount),
								  to_u256!(0),
				)?;

			ensure!(supply_amount <= to_u256!(max_supply_amount), Error::<T>::UnacceptableSupplyAmount);

			lbp_info.supply_balance = to_balance!(
                badd(to_u256!(lbp_info.supply_balance), supply_amount).ok_or(ArithmeticError::Overflow)?
            )?;

			lbp_info.target_balance = to_balance!(
                bsub(to_u256!(lbp_info.target_balance), to_u256!(target_amount)).ok_or(ArithmeticError::Overflow)?
            )?;

			let spot_price_after =
				calc_spot_price(to_u256!(lbp_info.supply_balance),
								to_u256!(lbp_info.supply_weight),
								to_u256!(lbp_info.target_balance),
								to_u256!(lbp_info.target_weight),
								to_u256!(0),
				)?;

			ensure!(spot_price_after >= spot_price_before, Error::<T>::ErrMathApprox);
			ensure!(spot_price_after <= to_u256!(max_price), Error::<T>::UnacceptablePrice);
			ensure!(
                spot_price_before <= bdiv(supply_amount, to_u256!(target_amount)).ok_or(ArithmeticError::Overflow)?,
                Error::<T>::ErrMathApprox
            );

			let module_account_id = Self::account_id();

			T::Currency::transfer(supply_asset, &who,
								  &module_account_id, to_balance!(supply_amount)?)?;
			T::Currency::transfer(target_asset, &module_account_id,
								  &who, target_amount)?;

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(
				Event::Swapped(who, lbp_id, supply_asset,
							   target_asset, to_balance!(supply_amount)?, target_amount,
				)
			);

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn get_next_lbp_id() -> sp_std::result::Result<T::LbpId, DispatchErrorWithPostInfo> {
		let next_lbp_id = Self::next_lbp_id();
		let new_lbp_id = next_lbp_id.checked_add(&One::one()).ok_or(Error::<T>::NoLbpIdAvailable)?;
		NextLbpId::<T>::put(new_lbp_id);

		Ok(next_lbp_id)
	}

	fn ensure_weight(weight: u128) -> DispatchResultWithPostInfo {
		ensure!(weight >= MIN_WEIGHT, Error::<T>::ErrMinWeight);
		ensure!(weight <= MAX_WEIGHT, Error::<T>::ErrMaxWeight);

		Ok(().into())
	}

	fn put_price_history(lbp_id: T::LbpId, now: BlockNumber, supply_balance: Balance,
						 target_balance: Balance, supply_weight: u128, target_weight: u128) {
		let mut history = PriceHistory::<T>::get(lbp_id);
		history.push((now, supply_balance, target_balance, supply_weight, target_weight));
		PriceHistory::<T>::insert(lbp_id, history);
	}

	// Update weights in a predetermined way, between startBlock and endBlock.
	fn update_weights_gradually(block_number: T::BlockNumber) {
		let ongoing_lbps = OngoingLbps::<T>::iter_values().collect::<Vec<_>>();
		for (_, lbp_id) in ongoing_lbps {
			let _ = Self::update_weight(block_number, lbp_id);
		}
	}

	fn update_weight(
		block_number: T::BlockNumber,
		lbp_id: T::LbpId,
	) -> DispatchResultWithPostInfo {
		let now: BlockNumber = block_number.saturated_into();

		Lbps::<T>::try_mutate(lbp_id, |maybe_lbp_info| -> DispatchResultWithPostInfo {
			let lbp_info = maybe_lbp_info.as_mut().ok_or(Error::<T>::LbpNotFind)?;
			ensure!(lbp_info.status == LbpStatus::Pending ||
                lbp_info.status == LbpStatus::InProgress, Error::<T>::MustBeNotEndStatus);

			if lbp_info.status == LbpStatus::Pending && now >= lbp_info.start_block {
				lbp_info.status = LbpStatus::InProgress;
			}

			if now >= lbp_info.next_block && lbp_info.status == LbpStatus::InProgress {
				let supply_adjust_weight = calc_adjust_weight(
					to_u256!(lbp_info.initial_supply_start_weight),
					to_u256!(lbp_info.initial_supply_end_weight),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				).ok_or(ArithmeticError::Overflow)?;

				let target_adjust_weight = calc_adjust_weight(
					to_u256!(lbp_info.initial_target_start_weight),
					to_u256!(lbp_info.initial_target_end_weight),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				).ok_or(ArithmeticError::Overflow)?;

				lbp_info.supply_weight = to_balance!(supply_adjust_weight)?;
				lbp_info.target_weight = to_balance!(target_adjust_weight)?;

				Self::put_price_history(
					lbp_id,
					now,
					lbp_info.supply_balance,
					lbp_info.target_balance,
					lbp_info.supply_weight,
					lbp_info.target_weight,
				);

				lbp_info.step = lbp_info.step.checked_add(u32::one())
					.ok_or(ArithmeticError::Overflow)?;

				if lbp_info.step == lbp_info.steps + 1 {
					lbp_info.status = LbpStatus::Finished;
					return Ok(().into());
				}

				let next_adjust_block = calc_adjust_block(
					to_u256!(lbp_info.start_block),
					to_u256!(lbp_info.end_block),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				).ok_or(ArithmeticError::Overflow)?;

				ensure!(next_adjust_block <= to_u256!(BlockNumber::MAX), ArithmeticError::Overflow);

				lbp_info.next_block = next_adjust_block.as_u32();
			}

			Ok(().into())
		})
	}
}
