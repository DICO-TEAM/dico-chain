//! # Lbp Pallet
//!
//! ## Overview
//!
//! Lbp pallet provides functionality for managing liquidity bootstrapping pool and executing
//! trades.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(deprecated)]
#![allow(clippy::upper_case_acronyms)]

use core::convert::TryFrom;
use dico_primitives::{to_balance, to_u256};
use dico_primitives::{Amount, AssetId, Balance, BlockNumber};
use frame_support::{
	dispatch::DispatchErrorWithPostInfo,
	ensure,
	pallet_prelude::*,
	sp_runtime::traits::{AtLeast32Bit, CheckedAdd, One, Zero},
	// log,
	traits::Get,
	transactional,
	PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, SaturatedConversion},
	ArithmeticError, RuntimeDebug,
};
use sp_std::{vec, vec::Vec};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};

use pallet_dico_treasury::traits::DicoTreasuryHandler;
use frame_support::weights::Weight;

const KICO_BASE_WEIGHT: Weight = Weight::from_ref_time(20_0000_0000);

// mod benchmarking;
//
// pub mod weights;
//
// use weights::WeightInfo;

mod math;

use math::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LbpStatus {
	Pending,
	Cancelled,
	InProgress,
	Finished,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LbpInfo<AccountId> {
	pub start_block: BlockNumber,
	pub end_block: BlockNumber,
	pub steps: u32,
	pub owner: AccountId,
	// available-for-sale asset
	pub afs_asset: AssetId,
	// project fundraising asset
	pub fundraising_asset: AssetId,
	pub initial_afs_balance: Balance,
	pub initial_fundraising_balance: Balance,
	pub initial_afs_start_weight: u128,
	pub initial_afs_end_weight: u128,
	pub initial_fundraising_start_weight: u128,
	pub initial_fundraising_end_weight: u128,

	pub afs_balance: Balance,
	pub fundraising_balance: Balance,
	pub afs_weight: u128,
	pub fundraising_weight: u128,
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
		afs_asset: AssetId,
		fundraising_asset: AssetId,
		initial_afs_balance: Balance,
		initial_fundraising_balance: Balance,
		initial_afs_start_weight: u128,
		initial_afs_end_weight: u128,
		initial_fundraising_start_weight: u128,
		initial_fundraising_end_weight: u128,
	) -> Self {
		Self {
			start_block,
			end_block,
			steps,
			owner,
			afs_asset,
			fundraising_asset,
			initial_afs_balance,
			initial_fundraising_balance,
			initial_afs_start_weight,
			initial_afs_end_weight,
			initial_fundraising_start_weight,
			initial_fundraising_end_weight,
			afs_balance: initial_afs_balance,
			fundraising_balance: initial_fundraising_balance,
			afs_weight: initial_afs_start_weight,
			fundraising_weight: initial_fundraising_start_weight,
			status: LbpStatus::Pending,
			step: 0u32,
			next_block: BlockNumber::zero(),
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LbpPair(pub AssetId, pub AssetId);

impl LbpPair {
	pub fn new(asset_a: AssetId, asset_b: AssetId) -> Self {
		if asset_a < asset_b {
			Self(asset_a, asset_b)
		} else {
			Self(asset_b, asset_a)
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
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			Self::update_weights_gradually(now);
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Balance = Balance, Amount = Amount>;

		/// The LBP's module id, keep all assets in LBP.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The mining pool id
		type LbpId: Parameter + Member + Into<u32> + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;

		// /// Weight information for the extrinsics in this module.
		// type WeightInfo: WeightInfo;

		type TreasuryHandler: DicoTreasuryHandler<Self::AccountId>;

		/// The origin that is allowed to set or update parameter.
		type FounderSetOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::error]
	pub enum Error<T> {
		MustBeDifferentAsset,
		NoLbpIdAvailable,
		LbpNotFind,
		OngoingLbpNotFind,
		LbpPairOngoing,
		MustBeOwner,
		MustBeNonTradingStatus,
		MustBeNotEndStatus,
		MustBeInProgressStatus,
		InvalidFundraisingAsset,
		InvalidFundraisingAmount,
		InvalidTargetAmount,
		UnacceptableSupplyAmount,
		UnacceptableTargetAmount,
		ErrMathApprox,
		ErrMinSteps,
		ErrMaxSteps,
		ErrMinWeight,
		ErrMaxWeight,
		ErrStartEndBlock,
		ErrMaxDurationBlock,
		ErrMinDurationBlock,
		StartBlockOutDate,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		FundraisingAssetAdded(AssetId, Balance),
		FundraisingAssetRemoved(AssetId),
		LbpCreated(T::AccountId, T::LbpId, AssetId, AssetId, Balance, Balance),
		LbpExited(T::AccountId, T::LbpId),
		Swapped(T::AccountId, T::LbpId, AssetId, AssetId, Balance, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn next_lbp_id)]
	pub type NextLbpId<T: Config> = StorageValue<_, T::LbpId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_lbp)]
	pub type Lbps<T: Config> = StorageMap<_, Blake2_128Concat, T::LbpId, LbpInfo<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_ongoing_lbp)]
	pub type OngoingLbps<T: Config> = StorageMap<_, Blake2_128Concat, LbpPair, (T::AccountId, T::LbpId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_all_sfa)]
	pub type SupportFundraisingAssets<T: Config> = StorageValue<_, Vec<(AssetId, Balance)>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_history)]
	pub type PriceHistory<T: Config> =
		StorageMap<_, Blake2_128Concat, T::LbpId, Vec<(BlockNumber, Balance, Balance, u128, u128)>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn add_fundraising_asset(
			origin: OriginFor<T>,
			fundraising: AssetId,
			min_fundraising_amount: Balance,
		) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;
			Self::add_fundraising(fundraising, min_fundraising_amount);
			Self::deposit_event(Event::FundraisingAssetAdded(fundraising, min_fundraising_amount));
			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn remove_fundraising_asset(origin: OriginFor<T>, fundraising: AssetId) -> DispatchResultWithPostInfo {
			T::FounderSetOrigin::ensure_origin(origin)?;
			Self::remove_fundraising(fundraising);
			Self::deposit_event(Event::FundraisingAssetRemoved(fundraising));
			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn create_lbp(
			origin: OriginFor<T>,
			afs_asset: AssetId,
			fundraising_asset: AssetId,
			afs_balance: Balance,
			fundraising_balance: Balance,
			afs_start_weight: u128,
			afs_end_weight: u128,
			fundraising_start_weight: u128,
			fundraising_end_weight: u128,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			steps: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(afs_asset != fundraising_asset, Error::<T>::MustBeDifferentAsset);
			ensure!(steps >= MIN_STEPS, Error::<T>::ErrMinSteps);
			ensure!(steps <= MAX_STEPS, Error::<T>::ErrMaxSteps);
			ensure!(start_block < end_block, Error::<T>::ErrStartEndBlock);
			Self::ensure_fundraising(fundraising_asset, fundraising_balance)?;

			let initial_start_block: BlockNumber = start_block.saturated_into();
			let initial_end_block: BlockNumber = end_block.saturated_into();
			let duration_block = initial_end_block
				.checked_sub(initial_start_block)
				.ok_or(ArithmeticError::Overflow)?;
			ensure!(duration_block <= MAX_DURATION_BLOCK, Error::<T>::ErrMaxDurationBlock);
			ensure!(duration_block >= MIN_DURATION_BLOCK, Error::<T>::ErrMinDurationBlock);

			Self::ensure_weight(afs_start_weight)?;
			Self::ensure_weight(afs_end_weight)?;
			Self::ensure_weight(fundraising_start_weight)?;
			Self::ensure_weight(fundraising_end_weight)?;

			let lbp_pair = LbpPair::new(afs_asset, fundraising_asset);
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
				afs_asset,
				fundraising_asset,
				afs_balance,
				fundraising_balance,
				afs_start_weight,
				afs_end_weight,
				fundraising_start_weight,
				fundraising_end_weight,
			);

			if block_number == initial_start_block {
				lbp_info.status = LbpStatus::InProgress;
			}

			let module_account_id = Self::account_id();

			T::Currency::transfer(afs_asset, &who, &module_account_id, afs_balance)?;
			T::Currency::transfer(fundraising_asset, &who, &module_account_id, fundraising_balance)?;

			Lbps::<T>::insert(lbp_id, lbp_info);
			OngoingLbps::<T>::insert(lbp_pair, (who.clone(), lbp_id));

			Self::deposit_event(Event::LbpCreated(
				who,
				lbp_id,
				afs_asset,
				fundraising_asset,
				afs_balance,
				fundraising_balance,
			));

			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn exit_lbp(origin: OriginFor<T>, lbp_id: T::LbpId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut lbp_info = Lbps::<T>::get(lbp_id).ok_or(Error::<T>::LbpNotFind)?;

			ensure!(lbp_info.owner == who, Error::<T>::MustBeOwner);
			ensure!(
				lbp_info.status == LbpStatus::Pending || lbp_info.status == LbpStatus::Finished,
				Error::<T>::MustBeNonTradingStatus
			);

			let module_account_id = Self::account_id();

			T::Currency::transfer(
				lbp_info.afs_asset.clone(),
				&module_account_id,
				&who,
				lbp_info.afs_balance,
			)?;
			T::Currency::transfer(
				lbp_info.fundraising_asset.clone(),
				&module_account_id,
				&who,
				lbp_info.fundraising_balance,
			)?;

			lbp_info.afs_balance = Balance::zero();
			lbp_info.fundraising_balance = Balance::zero();
			if lbp_info.status == LbpStatus::Pending {
				lbp_info.status = LbpStatus::Cancelled;
			}

			let lbp_pair = LbpPair::new(lbp_info.afs_asset.clone(), lbp_info.fundraising_asset.clone());
			OngoingLbps::<T>::remove(lbp_pair);

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(Event::LbpExited(who, lbp_id));

			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn swap_exact_amount_supply(
			origin: OriginFor<T>,
			supply_asset: AssetId,
			supply_amount: Balance,
			target_asset: AssetId,
			min_target_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let lbp_pair = LbpPair::new(supply_asset, target_asset);
			let ongoing_lbp = OngoingLbps::<T>::try_get(lbp_pair).map_err(|_| Error::<T>::OngoingLbpNotFind)?;

			let lbp_id = ongoing_lbp.1;
			let mut lbp_info = Lbps::<T>::get(lbp_id).ok_or(Error::<T>::LbpNotFind)?;

			ensure!(
				lbp_info.status == LbpStatus::InProgress,
				Error::<T>::MustBeInProgressStatus
			);

			let (crowdfunding_amount, target_amount) = if supply_asset == lbp_info.afs_asset {
				let (crowdfunding_amount, target_amount) = Self::get_amount_out(
					lbp_info.afs_balance,
					lbp_info.afs_weight,
					lbp_info.fundraising_balance,
					lbp_info.fundraising_weight,
					supply_amount,
					min_target_amount,
				)?;

				lbp_info.afs_balance =
					to_balance!(badd(to_u256!(lbp_info.afs_balance), to_u256!(crowdfunding_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				lbp_info.fundraising_balance =
					to_balance!(bsub(to_u256!(lbp_info.fundraising_balance), to_u256!(target_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				(crowdfunding_amount, target_amount)
			} else {
				// supply_asset = lbp_info.fundraising_asset
				let (crowdfunding_amount, target_amount) = Self::get_amount_out(
					lbp_info.fundraising_balance,
					lbp_info.fundraising_weight,
					lbp_info.afs_balance,
					lbp_info.afs_weight,
					supply_amount,
					min_target_amount,
				)?;

				lbp_info.fundraising_balance = to_balance!(badd(
					to_u256!(lbp_info.fundraising_balance),
					to_u256!(crowdfunding_amount)
				)
				.ok_or(ArithmeticError::Overflow)?)?;

				lbp_info.afs_balance =
					to_balance!(bsub(to_u256!(lbp_info.afs_balance), to_u256!(target_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				(crowdfunding_amount, target_amount)
			};

			let module_account_id = Self::account_id();

			T::Currency::transfer(supply_asset, &who, &module_account_id, crowdfunding_amount)?;
			T::Currency::transfer(target_asset, &module_account_id, &who, target_amount)?;

			let treasury_account_id = T::TreasuryHandler::get_treasury_account_id();
			let crowdfunding_fee = supply_amount
				.checked_sub(crowdfunding_amount)
				.ok_or(ArithmeticError::Overflow)?;
			T::Currency::transfer(supply_asset, &who, &treasury_account_id, crowdfunding_fee)?;

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(Event::Swapped(
				who,
				lbp_id,
				supply_asset,
				target_asset,
				supply_amount,
				target_amount,
			));

			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		#[transactional]
		pub fn swap_exact_amount_target(
			origin: OriginFor<T>,
			supply_asset: AssetId,
			max_supply_amount: Balance,
			target_asset: AssetId,
			target_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let lbp_pair = LbpPair::new(supply_asset, target_asset);
			let ongoing_lbp = OngoingLbps::<T>::try_get(lbp_pair).map_err(|_| Error::<T>::OngoingLbpNotFind)?;

			let lbp_id = ongoing_lbp.1;
			let mut lbp_info = Lbps::<T>::get(lbp_id).ok_or(Error::<T>::LbpNotFind)?;

			ensure!(
				lbp_info.status == LbpStatus::InProgress,
				Error::<T>::MustBeInProgressStatus
			);

			let (crowdfunding_amount, supply_amount) = if supply_asset == lbp_info.afs_asset {
				let (crowdfunding_amount, supply_amount) = Self::get_amount_in(
					lbp_info.afs_balance,
					lbp_info.afs_weight,
					lbp_info.fundraising_balance,
					lbp_info.fundraising_weight,
					target_amount,
					max_supply_amount,
				)?;

				lbp_info.afs_balance =
					to_balance!(badd(to_u256!(lbp_info.afs_balance), to_u256!(crowdfunding_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				lbp_info.fundraising_balance =
					to_balance!(bsub(to_u256!(lbp_info.fundraising_balance), to_u256!(target_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				(crowdfunding_amount, supply_amount)
			} else {
				// supply_asset = lbp_info.fundraising_asset
				let (crowdfunding_amount, supply_amount) = Self::get_amount_in(
					lbp_info.fundraising_balance,
					lbp_info.fundraising_weight,
					lbp_info.afs_balance,
					lbp_info.afs_weight,
					target_amount,
					max_supply_amount,
				)?;

				lbp_info.fundraising_balance = to_balance!(badd(
					to_u256!(lbp_info.fundraising_balance),
					to_u256!(crowdfunding_amount)
				)
				.ok_or(ArithmeticError::Overflow)?)?;

				lbp_info.afs_balance =
					to_balance!(bsub(to_u256!(lbp_info.afs_balance), to_u256!(target_amount))
						.ok_or(ArithmeticError::Overflow)?)?;

				(crowdfunding_amount, supply_amount)
			};

			let module_account_id = Self::account_id();

			T::Currency::transfer(
				supply_asset,
				&who,
				&module_account_id,
				to_balance!(crowdfunding_amount)?,
			)?;
			T::Currency::transfer(target_asset, &module_account_id, &who, target_amount)?;

			let treasury_account_id = T::TreasuryHandler::get_treasury_account_id();
			let crowdfunding_fee = supply_amount
				.checked_sub(crowdfunding_amount)
				.ok_or(ArithmeticError::Overflow)?;
			T::Currency::transfer(supply_asset, &who, &treasury_account_id, crowdfunding_fee)?;

			Lbps::<T>::insert(lbp_id, lbp_info);

			Self::deposit_event(Event::Swapped(
				who,
				lbp_id,
				supply_asset,
				target_asset,
				supply_amount,
				target_amount,
			));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	fn get_next_lbp_id() -> sp_std::result::Result<T::LbpId, DispatchErrorWithPostInfo> {
		let next_lbp_id = Self::next_lbp_id();
		let new_lbp_id = next_lbp_id
			.checked_add(&One::one())
			.ok_or(Error::<T>::NoLbpIdAvailable)?;
		NextLbpId::<T>::put(new_lbp_id);

		Ok(next_lbp_id)
	}

	fn ensure_fundraising(
		fundraising: AssetId,
		fundraising_amount: Balance,
	) -> sp_std::result::Result<(), DispatchErrorWithPostInfo> {
		let sfa = SupportFundraisingAssets::<T>::get().ok_or(Error::<T>::InvalidFundraisingAsset)?;
		for (asset, min_fundraising_amount) in sfa.into_iter() {
			if asset == fundraising {
				ensure!(
					fundraising_amount >= min_fundraising_amount,
					Error::<T>::InvalidFundraisingAmount
				);
				return Ok(());
			}
		}

		Err(Error::<T>::InvalidFundraisingAsset)?
	}

	fn add_fundraising(fundraising: AssetId, min_fundraising_amount: Balance) {
		let sfa = SupportFundraisingAssets::<T>::get()
			.and_then(|mut x| {
				for (asset, amount) in x.iter_mut() {
					if *asset == fundraising && *amount != min_fundraising_amount {
						*amount = min_fundraising_amount;
					}
				}
				Some(x)
			})
			.and_then(|mut x| {
				if !x.contains(&(fundraising, min_fundraising_amount)) {
					x.push((fundraising, min_fundraising_amount));
				}
				Some(x)
			})
			.unwrap_or(vec![(fundraising, min_fundraising_amount)]);

		SupportFundraisingAssets::<T>::put(sfa);
	}

	fn remove_fundraising(fundraising: AssetId) {
		let sfa = SupportFundraisingAssets::<T>::get().and_then(|mut x| {
			x.retain(|x| x.0 != fundraising);
			return if !x.is_empty() { Some(x) } else { None };
		});

		if let Some(val) = sfa {
			SupportFundraisingAssets::<T>::put(val);
		} else {
			SupportFundraisingAssets::<T>::kill();
		}
	}

	fn ensure_weight(weight: u128) -> DispatchResultWithPostInfo {
		ensure!(weight >= MIN_WEIGHT, Error::<T>::ErrMinWeight);
		ensure!(weight <= MAX_WEIGHT, Error::<T>::ErrMaxWeight);

		Ok(().into())
	}

	fn insert_price_history(
		lbp_id: T::LbpId,
		now: BlockNumber,
		supply_balance: Balance,
		target_balance: Balance,
		supply_weight: u128,
		target_weight: u128,
	) {
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

	fn get_amount_out(
		asset_balance_in: Balance,
		asset_weight_in: u128,
		asset_balance_out: Balance,
		asset_weight_out: u128,
		supply_amount: Balance,
		min_target_amount: Balance,
	) -> sp_std::result::Result<(Balance, Balance), DispatchErrorWithPostInfo> {
		ensure!(min_target_amount <= asset_balance_out, Error::<T>::InvalidTargetAmount);

		let crowdfunding_amount = calc_crowdfunding_amount(to_u256!(supply_amount), to_u256!(math::CROWDFUNDING_FEE))
			.ok_or(ArithmeticError::Overflow)?;

		let spot_price_before = calc_spot_price(
			to_u256!(asset_balance_in),
			to_u256!(asset_weight_in),
			to_u256!(asset_balance_out),
			to_u256!(asset_weight_out),
			to_u256!(SWAP_FEE),
		)?;

		let target_amount = calc_out_given_in(
			to_u256!(asset_balance_in),
			to_u256!(asset_weight_in),
			to_u256!(asset_balance_out),
			to_u256!(asset_weight_out),
			to_u256!(crowdfunding_amount),
			to_u256!(SWAP_FEE),
		)?;

		ensure!(
			target_amount >= to_u256!(min_target_amount),
			Error::<T>::UnacceptableTargetAmount
		);
		ensure!(
			spot_price_before <= bdiv(to_u256!(crowdfunding_amount), target_amount).ok_or(ArithmeticError::Overflow)?,
			Error::<T>::ErrMathApprox
		);

		let new_asset_balance_in =
			badd(to_u256!(asset_balance_in), to_u256!(crowdfunding_amount)).ok_or(ArithmeticError::Overflow)?;
		let new_asset_balance_out =
			bsub(to_u256!(asset_balance_out), target_amount).ok_or(ArithmeticError::Overflow)?;

		let spot_price_after = calc_spot_price(
			new_asset_balance_in,
			to_u256!(asset_weight_in),
			new_asset_balance_out,
			to_u256!(asset_weight_out),
			to_u256!(SWAP_FEE),
		)?;

		ensure!(spot_price_after >= spot_price_before, Error::<T>::ErrMathApprox);

		let crowdfunding_amount = to_balance!(crowdfunding_amount)?;
		let target_balance = to_balance!(target_amount)?;
		Ok((crowdfunding_amount, target_balance))
	}

	fn get_amount_in(
		asset_balance_in: Balance,
		asset_weight_in: u128,
		asset_balance_out: Balance,
		asset_weight_out: u128,
		target_amount: Balance,
		max_supply_amount: Balance,
	) -> sp_std::result::Result<(Balance, Balance), DispatchErrorWithPostInfo> {
		ensure!(target_amount <= asset_balance_out, Error::<T>::InvalidTargetAmount);

		let spot_price_before = calc_spot_price(
			to_u256!(asset_balance_in),
			to_u256!(asset_weight_in),
			to_u256!(asset_balance_out),
			to_u256!(asset_weight_out),
			to_u256!(SWAP_FEE),
		)?;

		let crowdfunding_amount = calc_in_given_out(
			to_u256!(asset_balance_in),
			to_u256!(asset_weight_in),
			to_u256!(asset_balance_out),
			to_u256!(asset_weight_out),
			to_u256!(target_amount),
			to_u256!(SWAP_FEE),
		)?;

		let supply_amount =
			calc_supply_amount_with_fee(to_u256!(crowdfunding_amount), to_u256!(math::CROWDFUNDING_FEE))
				.ok_or(ArithmeticError::Overflow)?;

		ensure!(supply_amount >= crowdfunding_amount, Error::<T>::ErrMathApprox);
		ensure!(
			supply_amount <= to_u256!(max_supply_amount),
			Error::<T>::UnacceptableSupplyAmount
		);

		let new_asset_balance_in =
			badd(to_u256!(asset_balance_in), crowdfunding_amount).ok_or(ArithmeticError::Overflow)?;
		let new_asset_balance_out =
			bsub(to_u256!(asset_balance_out), to_u256!(target_amount)).ok_or(ArithmeticError::Overflow)?;

		let spot_price_after = calc_spot_price(
			new_asset_balance_in,
			to_u256!(asset_weight_in),
			new_asset_balance_out,
			to_u256!(asset_weight_out),
			to_u256!(SWAP_FEE),
		)?;

		ensure!(spot_price_after >= spot_price_before, Error::<T>::ErrMathApprox);
		ensure!(
			spot_price_before <= bdiv(crowdfunding_amount, to_u256!(target_amount)).ok_or(ArithmeticError::Overflow)?,
			Error::<T>::ErrMathApprox
		);

		let crowdfunding_amount = to_balance!(crowdfunding_amount)?;
		let supply_balance = to_balance!(supply_amount)?;
		Ok((crowdfunding_amount, supply_balance))
	}

	fn update_weight(block_number: T::BlockNumber, lbp_id: T::LbpId) -> DispatchResultWithPostInfo {
		let now: BlockNumber = block_number.saturated_into();

		Lbps::<T>::try_mutate(lbp_id, |maybe_lbp_info| -> DispatchResultWithPostInfo {
			let lbp_info = maybe_lbp_info.as_mut().ok_or(Error::<T>::LbpNotFind)?;
			ensure!(
				lbp_info.status == LbpStatus::Pending || lbp_info.status == LbpStatus::InProgress,
				Error::<T>::MustBeNotEndStatus
			);

			if lbp_info.status == LbpStatus::Pending && now >= lbp_info.start_block {
				lbp_info.status = LbpStatus::InProgress;
			}

			if now >= lbp_info.next_block && lbp_info.status == LbpStatus::InProgress {
				let afs_adjust_weight = calc_adjust_weight(
					to_u256!(lbp_info.initial_afs_start_weight),
					to_u256!(lbp_info.initial_afs_end_weight),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				)
				.ok_or(ArithmeticError::Overflow)?;

				let fundraising_adjust_weight = calc_adjust_weight(
					to_u256!(lbp_info.initial_fundraising_start_weight),
					to_u256!(lbp_info.initial_fundraising_end_weight),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				)
				.ok_or(ArithmeticError::Overflow)?;

				lbp_info.afs_weight = to_balance!(afs_adjust_weight)?;
				lbp_info.fundraising_weight = to_balance!(fundraising_adjust_weight)?;

				Self::insert_price_history(
					lbp_id,
					now,
					lbp_info.afs_balance,
					lbp_info.fundraising_balance,
					lbp_info.afs_weight,
					lbp_info.fundraising_weight,
				);

				lbp_info.step = lbp_info.step.checked_add(u32::one()).ok_or(ArithmeticError::Overflow)?;

				if lbp_info.step == lbp_info.steps + 1 {
					lbp_info.status = LbpStatus::Finished;
					return Ok(().into());
				}

				let next_adjust_block = calc_adjust_block(
					to_u256!(lbp_info.start_block),
					to_u256!(lbp_info.end_block),
					to_u256!(lbp_info.steps),
					to_u256!(lbp_info.step),
				)
				.ok_or(ArithmeticError::Overflow)?;

				ensure!(
					next_adjust_block <= to_u256!(BlockNumber::MAX),
					ArithmeticError::Overflow
				);

				lbp_info.next_block = next_adjust_block.as_u32();
			}

			Ok(().into())
		})
	}
}
