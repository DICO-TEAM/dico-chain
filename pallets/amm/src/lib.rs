//! # AMM Pallet
//!
//! ## Overview
//!
//! AMM pallet provides functionality for managing liquidity pool and executing trades.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use core::convert::TryFrom;
use frame_support::{
	ensure, traits::Get, transactional, PalletId,
	sp_runtime::traits::{Zero, One},
	pallet_prelude::*,
	pallet_prelude::DispatchResultWithPostInfo,
	dispatch::DispatchErrorWithPostInfo,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use dico_primitives::{AssetId, Balance, Amount, to_u256, to_balance};
use sp_std::{vec::Vec};
use sp_runtime::{ArithmeticError, traits::{AccountIdConversion}, RuntimeDebug};
use sp_core::U256;
use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use dico_currencies::{DicoAssetMetadata};

pub mod math;
mod benchmarking;
pub mod weights;

use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Pair(pub AssetId, pub AssetId);

impl Pair {
	pub fn new(asset_a: AssetId, asset_b: AssetId) -> Self {
		if asset_a < asset_b { Self(asset_a, asset_b) } else { Self(asset_b, asset_a) }
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Default, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LiquidityInfo(pub Balance, pub Balance, pub AssetId);

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;
use dico_currencies::currencies_trait::CurrenciesHandler;
use crate::math::LIQUIDITY_DECIMALS;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config + dico_currencies::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type Currency: MultiCurrencyExtended<
			Self::AccountId, CurrencyId=AssetId, Balance=Balance, Amount=Amount
		>;

		#[pallet::constant]
		type LiquidityAssetIdBase: Get<AssetId>;

		/// The AMM's module id, keep all assets in AMM.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Must use different assets to add liquidity.
		MustBeDifferentAsset,
		/// Liquid asset can not add liquidity pool.
		MustBeNonLiquidAsset,
		/// The number of added liquid assets must be greater than 0.
		MustAddNonZeroAmount,
		/// The asset exchange path does not exist.
		InvalidSwapPath,
		/// Add too little liquidity.
		InsufficientMintLiquidity,
		/// add insufficient assets amount.
		InsufficientAmount,
		/// The amount of liquidity removed is 0.
		RemoveZeroLiquidity,
		/// Liquidity is not enough.
		InsufficientLiquidity,
		/// The assets obtained by removing the liquidity are too few for users to accept.
		UnacceptableLiquidityWithdrawn,
		/// The swap doesn't meet the invariant check.
		InvariantCheckFailed,
		/// The number of exchanged output assets is too small for users to accept.
		UnacceptableOutputAmount,
		/// Too many input assets are required for the exchange, and the user cannot accept it.
		UnacceptableInputAmount,
		/// The liquidity number has been exhausted.
		NoLiquidityIdAvailable,
		/// The liquidity pool does not exist.
		LiquidityNotFind,
		/// The asset metadata is invalid.
		AssetMetadataInvalid,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New liquidity was provided to the pool. [who, liquidity_id, asset a, asset b, amount a, amount b]
		LiquidityAdded(T::AccountId, AssetId, AssetId, AssetId, Balance, Balance),

		/// Liquidity was removed from the pool. [who, liquidity_id, asset a, asset b, liquidity]
		LiquidityRemoved(T::AccountId, AssetId, AssetId, AssetId, Balance),

		/// Swap asset cross path. [who, path, amount in, amount out]
		Swapped(T::AccountId, Vec<AssetId>, Balance, Balance),

		/// Deposit test asset.
		AssetDeposited(T::AccountId, AssetId, Balance),
	}

	#[pallet::storage]
	#[pallet::getter(fn get_liquidity)]
	pub type Liquidity<T: Config> = StorageMap<_, Blake2_128Concat, Pair, LiquidityInfo, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_liquidity_id)]
	pub type NextLiquidityId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add liquidity to previously created asset pair pool.
		///
		/// Emits `LiquidityAdded` event when successful.
		#[pallet::weight(< T as Config >::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetId,
			asset_b: AssetId,
			amount_a_desired: Balance,
			amount_b_desired: Balance,
			amount_a_min: Balance,
			amount_b_min: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(asset_a != asset_b, Error::<T>::MustBeDifferentAsset);
			ensure!(amount_a_desired != 0 && amount_b_desired != 0, Error::<T>::MustAddNonZeroAmount);
			ensure!(
				asset_a <= T::LiquidityAssetIdBase::get() && asset_b <= T::LiquidityAssetIdBase::get(),
				Error::<T>::MustBeNonLiquidAsset
			);

			let pair = Self::pair_for(asset_a, asset_b);
			let mut liquidity_info = Liquidity::<T>::get(pair).unwrap_or_default();

			let (asset_0_amount, asset_1_amount, liquidity_id) =
				(&mut liquidity_info.0, &mut liquidity_info.1, &mut liquidity_info.2);

			let (reserve_a, reserve_b) = Self::get_reserves(asset_a, asset_b).unwrap_or((0u128, 0u128));

			let amount_in = math::calc_amount_in(
				to_u256!(reserve_a),
				to_u256!(reserve_b),
				to_u256!(amount_a_desired),
				to_u256!(amount_b_desired),
				to_u256!(amount_a_min),
				to_u256!(amount_b_min),
			)?;
			let (amount_a, amount_b) = amount_in.ok_or(Error::<T>::InsufficientAmount)?;
			let (amount_a, amount_b) = (to_balance!(amount_a)?, to_balance!(amount_b)?);

			let module_account_id = Self::account_id();

			if liquidity_id.is_zero() {
				*liquidity_id = Self::create_liquidity_asset(asset_a, asset_b)?;
			}

			let new_liquidity_id = *liquidity_id;

			let total_liquidity: Balance = T::Currency::total_issuance(*liquidity_id);

			if total_liquidity.is_zero() {
				// permanently lock the first MINIMUM_LIQUIDITY assets
				T::Currency::deposit(*liquidity_id, &module_account_id, math::MINIMUM_LIQUIDITY)?;
			}

			let add_liquidity = math::calc_liquidity_add(
				to_u256!(reserve_a),
				to_u256!(reserve_b),
				to_u256!(amount_a),
				to_u256!(amount_b),
				to_u256!(total_liquidity),
				to_u256!(math::MINIMUM_LIQUIDITY),
			)?;
			let add_liquidity = to_balance!(add_liquidity)?;

			ensure!(add_liquidity > Balance::zero(), Error::<T>::InsufficientMintLiquidity);

			T::Currency::transfer(asset_a, &who, &module_account_id, amount_a)?;
			T::Currency::transfer(asset_b, &who, &module_account_id, amount_b)?;

			T::Currency::deposit(*liquidity_id, &who, add_liquidity)?;

			let (add_amount_0, add_amount_1) = if pair.0 == asset_a {
				(amount_a, amount_b)
			} else {
				(amount_b, amount_a)
			};

			*asset_0_amount = asset_0_amount
				.checked_add(add_amount_0).ok_or(ArithmeticError::Overflow)?;
			*asset_1_amount = asset_1_amount
				.checked_add(add_amount_1).ok_or(ArithmeticError::Overflow)?;

			Liquidity::<T>::insert(pair, liquidity_info);

			Self::deposit_event(
				Event::LiquidityAdded(
					who,
					new_liquidity_id,
					asset_a,
					asset_b,
					amount_a,
					amount_b,
				)
			);

			Ok(().into())
		}


		/// Remove liquidity from specific liquidity pool in the form of burning shares.
		///
		/// Emits 'LiquidityRemoved' when successful.
		#[pallet::weight(< T as Config >::WeightInfo::remove_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetId,
			asset_b: AssetId,
			remove_liquidity: Balance,
			amount_a_min: Balance,
			amount_b_min: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let pair = Self::pair_for(asset_a, asset_b);

			Liquidity::<T>::try_mutate(pair, |maybe_liquidity_info| -> DispatchResultWithPostInfo {
				let liquidity_info = maybe_liquidity_info.as_mut().ok_or(Error::<T>::LiquidityNotFind)?;

				let (asset_0_amount, asset_1_amount, liquidity_id) =
					(&mut liquidity_info.0, &mut liquidity_info.1, &mut liquidity_info.2);

				ensure!(!remove_liquidity.is_zero(), Error::<T>::RemoveZeroLiquidity);

				let total_liquidity: Balance = T::Currency::total_issuance(*liquidity_id);

				ensure!(total_liquidity >= remove_liquidity, Error::<T>::InsufficientLiquidity);

				let (reserve_a, reserve_b) = Self::get_reserves(asset_a, asset_b)?;
				let (remove_amount_a, remove_amount_b) = math::calc_amount_out(
					to_u256!(reserve_a),
					to_u256!(reserve_b),
					to_u256!(remove_liquidity),
					to_u256!(total_liquidity),
				)?;

				let remove_amount_a = to_balance!(remove_amount_a)?;
				let remove_amount_b = to_balance!(remove_amount_b)?;

				ensure!(
					remove_amount_a >= amount_a_min && remove_amount_b >= amount_b_min,
					Error::<T>::UnacceptableLiquidityWithdrawn,
				);

				let module_account_id = Self::account_id();

				T::Currency::transfer(asset_a, &module_account_id, &who, remove_amount_a)?;
				T::Currency::transfer(asset_b, &module_account_id, &who, remove_amount_b)?;

				T::Currency::withdraw(*liquidity_id, &who, remove_liquidity)?;

				let (remove_amount_0, remove_amount_1) = if pair.0 == asset_a {
					(remove_amount_a, remove_amount_b)
				} else {
					(remove_amount_b, remove_amount_a)
				};

				*asset_0_amount = asset_0_amount.checked_sub(remove_amount_0).ok_or(ArithmeticError::Overflow)?;
				*asset_1_amount = asset_1_amount.checked_sub(remove_amount_1).ok_or(ArithmeticError::Overflow)?;

				Self::deposit_event(
					Event::LiquidityRemoved(
						who,
						*liquidity_id,
						asset_a,
						asset_b,
						remove_liquidity,
					)
				);

				Ok(().into())
			})
		}


		/// Use a fixed amount of supply assets to exchange for target assets not less than `amount_out_min`.
		///
		/// Emits 'Swapped' when successful.
		#[pallet::weight(< T as Config >::WeightInfo::swap_exact_assets_for_assets())]
		#[transactional]
		pub fn swap_exact_assets_for_assets(
			origin: OriginFor<T>,
			amount_in: Balance,
			amount_out_min: Balance,
			path: Vec<AssetId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let amounts = Self::get_amounts_out(amount_in, &path)?;

			let amount_out = amounts[amounts.len() - 1];
			ensure!(amount_out >= amount_out_min, Error::<T>::UnacceptableOutputAmount);
			let module_account_id = Self::account_id();

			T::Currency::transfer(path[0], &who, &module_account_id, amount_in)?;
			Self::swap(&amounts, &path)?;
			T::Currency::transfer(path[path.len() - 1], &module_account_id, &who, amount_out)?;

			Self::deposit_event(
				Event::Swapped(
					who,
					path,
					amount_in,
					amount_out,
				)
			);

			Ok(().into())
		}


		/// Use no more than `amount_in_max` supply assets to exchange for a fixed amount of target assets.
		///
		/// Emits 'Swapped' when successful.
		#[pallet::weight(< T as Config >::WeightInfo::swap_assets_for_exact_assets())]
		#[transactional]
		pub fn swap_assets_for_exact_assets(
			origin: OriginFor<T>,
			amount_out: Balance,
			amount_in_max: Balance,
			path: Vec<AssetId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let amounts = Self::get_amounts_in(amount_out, &path)?;
			let amount_in = amounts[0];

			ensure!(amount_in <= amount_in_max, Error::<T>::UnacceptableInputAmount);

			let module_account_id = Self::account_id();
			T::Currency::transfer(path[0], &who, &module_account_id, amount_in)?;
			Self::swap(&amounts, &path)?;
			T::Currency::transfer(path[path.len() - 1], &module_account_id, &who, amount_out)?;

			Self::deposit_event(
				Event::Swapped(
					who,
					path,
					amount_in,
					amount_out,
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

	fn create_liquidity_asset(
		asset_a: AssetId,
		asset_b: AssetId,
	) -> sp_std::result::Result<AssetId, DispatchErrorWithPostInfo> {
		let asset_a_metadata = dico_currencies::module::Pallet::<T>::get_metadata(asset_a)?;
		let asset_b_metadata = dico_currencies::module::Pallet::<T>::get_metadata(asset_b)?;

		ensure!(
			!asset_a_metadata.name.is_empty()
			&& !asset_a_metadata.symbol.is_empty()
			&& !asset_b_metadata.name.is_empty()
			&& !asset_b_metadata.symbol.is_empty(),
			Error::<T>::AssetMetadataInvalid
		);

		let mut liquidity_symbol = asset_a_metadata.symbol.clone();
		liquidity_symbol.extend("-".bytes());
		liquidity_symbol.extend(asset_b_metadata.symbol.clone());

		let mut liquidity_name = "Amm ".as_bytes().to_vec();
		liquidity_name.extend(liquidity_symbol.clone());

		let module_account_id = Self::account_id();
		let new_liquidity_id = Self::get_next_liquidity_id()?;
		let amount =
			<<T as dico_currencies::Config>::MultiCurrency as
			orml_traits::MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance::from(0u32);

		let liquidity_metadata = DicoAssetMetadata {
			name: liquidity_name,
			symbol: liquidity_symbol,
			decimals: LIQUIDITY_DECIMALS,
		};

		dico_currencies::module::Pallet::<T>::do_create(
			module_account_id,
			new_liquidity_id,
			Some(liquidity_metadata),
			amount,
			true,
		)?;

		Ok(new_liquidity_id)
	}

	fn get_next_liquidity_id() -> sp_std::result::Result<AssetId, DispatchErrorWithPostInfo> {
		let mut next_liquidity_id = Self::next_liquidity_id();
		if next_liquidity_id.is_zero() {
			next_liquidity_id = T::LiquidityAssetIdBase::get();
		}

		let new_liquidity_id = next_liquidity_id
			.checked_add(One::one()).ok_or(Error::<T>::NoLiquidityIdAvailable)?;
		NextLiquidityId::<T>::put(new_liquidity_id);

		Ok(next_liquidity_id)
	}

	fn pair_for(asset_a: AssetId, asset_b: AssetId) -> Pair {
		Pair::new(asset_a, asset_b)
	}

	fn sort_asset(asset_a: AssetId, asset_b: AssetId) -> (AssetId, AssetId) {
		if asset_a < asset_b { (asset_a, asset_b) } else { (asset_b, asset_a) }
	}

	fn get_reserves(
		asset_a: AssetId,
		asset_b: AssetId,
	) -> sp_std::result::Result<(Balance, Balance), DispatchErrorWithPostInfo> {
		let (asset_0, _) = Self::sort_asset(asset_a, asset_b);
		let pair = Self::pair_for(asset_a, asset_b);

		let liquidity_info = Liquidity::<T>::try_get(pair)
			.map_err(|_| Error::<T>::LiquidityNotFind)?;

		if asset_a == asset_0 {
			Ok((liquidity_info.0, liquidity_info.1))
		} else {
			Ok((liquidity_info.1, liquidity_info.0))
		}
	}

	/// Performs a chained `get_amount_out` calculation for pairs of transactions of any path length.
	fn get_amounts_out(
		amount_in: Balance,
		path: &Vec<AssetId>,
	) -> sp_std::result::Result<Vec<Balance>, DispatchErrorWithPostInfo> {
		let path_len = path.len();
		ensure!(path_len >= 2, Error::<T>::InvalidSwapPath);

		let mut amounts = [0].repeat(path_len);
		amounts[0] = amount_in;

		for i in 0..path_len - 1 {
			let (reserve_in, reserve_out) = Self::get_reserves(path[i], path[i + 1])?;
			let amount = math::get_amount_out(
				to_u256!(amounts[i]),
				to_u256!(reserve_in),
				to_u256!(reserve_out),
			)?;

			amounts[i + 1] = to_balance!(amount)?;
		}

		Ok(amounts)
	}

	/// Performs a chained `get_amount_in` calculation for pairs of transactions of any path length.
	fn get_amounts_in(
		amount_out: Balance,
		path: &Vec<AssetId>,
	) -> sp_std::result::Result<Vec<Balance>, DispatchErrorWithPostInfo> {
		let path_len = path.len();
		ensure!(path_len >= 2, Error::<T>::InvalidSwapPath);

		let mut amounts = [0].repeat(path_len);
		amounts[path_len - 1] = amount_out;

		let mut i = path_len - 1;
		while i > 0 {
			let (reserve_in, reserve_out) = Self::get_reserves(path[i - 1], path[i])?;
			let amount = math::get_amount_in(
				to_u256!(amounts[i]),
				to_u256!(reserve_in),
				to_u256!(reserve_out),
			)?;

			amounts[i - 1] = to_balance!(amount)?;
			i -= 1;
		}

		Ok(amounts)
	}

	/// cross path swap, format: path = `[0, 1, 2]`, amounts = `[1000, 1, 1000]`
	fn swap(
		amounts: &Vec<Balance>,
		path: &Vec<AssetId>,
	) -> DispatchResultWithPostInfo {
		let path_len = path.len();

		for i in 0..path_len - 1 {
			let (asset_in, asset_out) = (path[i], path[i + 1]);
			let (amount_in, amount_out) = (amounts[i], amounts[i + 1]);
			let pair = Self::pair_for(asset_in, asset_out);

			Liquidity::<T>::try_mutate(pair, |maybe_liquidity_info| -> DispatchResultWithPostInfo {
				let liquidity_info = maybe_liquidity_info
					.as_mut().ok_or(Error::<T>::LiquidityNotFind)?;

				let (asset_0_amount, asset_1_amount, _liquidity_id) =
					(&mut liquidity_info.0, &mut liquidity_info.1, &mut liquidity_info.2);

				let invariant_before_swap: U256 = U256::from(*asset_0_amount)
					.saturating_mul(U256::from(*asset_1_amount));

				if pair.0 == asset_in {
					*asset_0_amount = asset_0_amount
						.checked_add(amount_in).ok_or(ArithmeticError::Overflow)?;
					*asset_1_amount = asset_1_amount
						.checked_sub(amount_out).ok_or(ArithmeticError::Overflow)?;
				} else {
					*asset_0_amount = asset_0_amount
						.checked_sub(amount_out).ok_or(ArithmeticError::Overflow)?;
					*asset_1_amount = asset_1_amount
						.checked_add(amount_in).ok_or(ArithmeticError::Overflow)?;
				}

				// invariant check to ensure the constant product formulas (k = x * y)
				let invariant_after_swap: U256 = U256::from(*asset_0_amount)
					.saturating_mul(U256::from(*asset_1_amount));
				ensure!(
					invariant_after_swap >= invariant_before_swap,
					Error::<T>::InvariantCheckFailed,
				);

				Ok(().into())
			})?;
		}

		Ok(().into())
	}
}
