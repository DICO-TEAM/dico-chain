// Copyright 2021 DICO  Developer.
// This file is part of DICO

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(unused_variables)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use daos_create_dao::{self as dao};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	pallet_prelude::*,
	traits::{
		Currency as PalletCurrency, ExistenceRequirement, Get, LockableCurrency as PalletLockableCurrency,
		ReservableCurrency as PalletReservableCurrency, WithdrawReasons,
	},
};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use orml_traits::{
	arithmetic::{Signed, SimpleArithmetic},
	// currency::TransferAll,
	BalanceStatus,
	BasicCurrency,
	BasicCurrencyExtended,
	BasicLockableCurrency,
	BasicReservableCurrency,
	LockIdentifier,
	MultiCurrency,
	MultiCurrencyExtended,
	MultiLockableCurrency,
	MultiReservableCurrency,
};
use pallet_vc::{self, Fee};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{CheckedSub, MaybeSerializeDeserialize, StaticLookup, Zero},
	Permill, RuntimeDebug,
};
use sp_std::boxed::Box;
use sp_std::vec;
use sp_std::vec::Vec;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker, result,
};
use xcm::{v1::MultiLocation, VersionedMultiLocation};
use frame_support::weights::Weight;

const KICO_BASE_WEIGHT: Weight = Weight::from_ref_time(20_0000_0000);

mod mock;

pub use module::*;

// mod benchmarking;
// pub mod weights;

pub mod currencies_trait;

// pub use weights::WeightInfo;

use currencies_trait::{AssetIdMapping, CurrenciesHandler};
pub use dico_primitives::{
	constants::{currency::*, time::*},
	AssetId,
};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct DicoAssetMetadata {
	/// project name
	pub name: Vec<u8>,
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub symbol: Vec<u8>,
	/// The number of decimals this asset uses to represent one unit.
	pub decimals: u8,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct DicoAssetInfo<AccountId, DicoAssetMetadata> {
	pub owner: AccountId,
	pub metadata: Option<DicoAssetMetadata>,
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type VcBalanceOf<T> =
		<<T as pallet_vc::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	// pub(crate) type AssetId =
	// <<T as Config>::MultiCurrency as MultiCurrency<<T as
	// frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type AmountOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrencyExtended<<T as frame_system::Config>::AccountId>>::Amount;

	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config + pallet_vc::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId>
			+ MultiCurrencyExtended<Self::AccountId>
			+ MultiLockableCurrency<Self::AccountId>
			+ MultiReservableCurrency<Self::AccountId>;

		type NativeCurrency: BasicCurrencyExtended<Self::AccountId, Balance = BalanceOf<Self>, Amount = AmountOf<Self>>
			+ BasicLockableCurrency<Self::AccountId, Balance = BalanceOf<Self>>
			+ BasicReservableCurrency<Self::AccountId, Balance = BalanceOf<Self>>;

		type SetLocationOrigin: EnsureOrigin<Self::Origin>;

		type ForceSetLocationOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		type GetNativeCurrencyId: Get<AssetId>;

		// /// Weight information for extrinsics in this module.
		// type WeightInfo: WeightInfo;

		type CreateConsume: Get<BalanceOf<Self>>;

		/// Maximum assets that can be created
		type MaxCreatableCurrencyId: Get<AssetId>;

		type USDCurrencyId: Get<AssetId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to convert the Amount type into Balance.
		AmountIntoBalanceFailed,
		/// Balance is too low.
		BalanceTooLow,
		/// Asset info is not exists
		AssetAlreadyExists,
		AssetNotExists,
		MetadataNotChange,
		MetadataErr,
		NotOwner,
		ShouldNotChangeDecimals,
		MetadataNotExists,
		NativeCurrency,
		CurrencyIdTooLarge,
		CurrencyIdTooLow,
		DaoExists,
		CexTransferClosed,
		AssetIdExisted,
		BadLocation,
		MultiLocationExisted,
		CrossTransferNotOpen,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency transfer success. [currency_id, from, to, amount]
		Transferred(AssetId, T::AccountId, T::AccountId, BalanceOf<T>),
		/// Update balance success. [currency_id, who, amount]
		BalanceUpdated(AssetId, T::AccountId, AmountOf<T>),
		/// Deposit success. [currency_id, who, amount]
		Deposited(AssetId, T::AccountId, BalanceOf<T>),
		/// Withdraw success. [currency_id, who, amount]
		Withdrawn(AssetId, T::AccountId, BalanceOf<T>),
		CreateAsset(T::AccountId, AssetId, BalanceOf<T>),
		SetMetadata(T::AccountId, AssetId, DicoAssetMetadata),
		Burn(T::AccountId, AssetId, BalanceOf<T>),
		SetLocation {
			currency_id: AssetId,
			location: MultiLocation,
		},
		ForceSetLocation {
			currency_id: AssetId,
			location: MultiLocation,
		},
		SetWeightRateMultiple {
			currency_id: AssetId,
			multiple: u128,
		},
		SetExistenialDepposit {
			currency_id: AssetId,
			existenial_deposit: BalanceOf<T>,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_info)]
	/// Metadata of an asset.
	pub type DicoAssetsInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, DicoAssetInfo<T::AccountId, DicoAssetMetadata>>;

	#[pallet::storage]
	#[pallet::getter(fn daos)]
	pub type Daos<T: Config> = StorageMap<_, Identity, AssetId, T::DaoId>;

	#[pallet::storage]
	#[pallet::getter(fn users_number)]
	pub type UsersNumber<T: Config> = StorageMap<_, Identity, AssetId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn existenial_deposits)]
	pub type ExistentialDeposits<T: Config> = StorageMap<_, Identity, AssetId, BalanceOf<T>, ValueQuery>;

	#[pallet::type_value]
	pub fn WeightRateMultipleOnEmpty<T: Config>() -> u128 {
		1000u128
	}

	#[pallet::storage]
	#[pallet::getter(fn weight_rate_multiple)]
	pub type WeightRateMultiple<T: Config> =
		StorageMap<_, Identity, AssetId, u128, ValueQuery, WeightRateMultipleOnEmpty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn asset_locations)]
	pub type AssetLocations<T: Config> = StorageMap<_, Twox64Concat, AssetId, MultiLocation, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn location_to_currency_ids)]
	pub type LocationToCurrencyIds<T: Config> = StorageMap<_, Twox64Concat, MultiLocation, AssetId, OptionQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub assets: Vec<(AssetId, DicoAssetInfo<T::AccountId, DicoAssetMetadata>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				assets: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.assets
				.iter()
				.for_each(|asset_info| DicoAssetsInfo::<T>::insert(asset_info.0, asset_info.1.clone()))
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Users create the asset.
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn create_asset(
			origin: OriginFor<T>,
			currency_id: AssetId,
			amount: BalanceOf<T>,
			metadata: Option<DicoAssetMetadata>,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;
			Self::do_create(user.clone(), currency_id, metadata, amount, false)?;

			Ok(().into())
		}

		/// After setting location, cross-chain transfers can be made
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn set_location(
			origin: OriginFor<T>,
			currency_id: AssetId,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResultWithPostInfo {
			T::SetLocationOrigin::ensure_origin(origin)?;
			let location: MultiLocation = (*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			ensure!(
				DicoAssetsInfo::<T>::contains_key(currency_id),
				Error::<T>::AssetNotExists
			);

			ensure!(
				!AssetLocations::<T>::contains_key(currency_id),
				Error::<T>::MultiLocationExisted
			);
			ensure!(
				!LocationToCurrencyIds::<T>::contains_key(location.clone()),
				Error::<T>::AssetIdExisted
			);

			AssetLocations::<T>::insert(currency_id, location.clone());
			LocationToCurrencyIds::<T>::insert(location.clone(), currency_id);
			Self::deposit_event(Event::SetLocation { currency_id, location });
			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn set_weight_rate_multiple(
			origin: OriginFor<T>,
			currency_id: AssetId,
			multiple: u128,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let info = DicoAssetsInfo::<T>::get(currency_id).ok_or(Error::<T>::AssetNotExists)?;
			ensure!(who == info.owner, Error::<T>::NotOwner);
			ensure!(
				AssetLocations::<T>::contains_key(currency_id),
				Error::<T>::CrossTransferNotOpen
			);
			WeightRateMultiple::<T>::insert(currency_id, multiple);
			Self::deposit_event(Event::SetWeightRateMultiple { currency_id, multiple });

			Ok(().into())
		}

		/// After setting location, cross-chain transfers can be made
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn force_set_location(
			origin: OriginFor<T>,
			currency_id: AssetId,
			location: Box<VersionedMultiLocation>,
		) -> DispatchResultWithPostInfo {
			T::ForceSetLocationOrigin::ensure_origin(origin)?;
			let location: MultiLocation = (*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			ensure!(
				DicoAssetsInfo::<T>::contains_key(currency_id),
				Error::<T>::AssetNotExists
			);
			if let Some(l) = AssetLocations::<T>::get(currency_id) {
				LocationToCurrencyIds::<T>::take(l);
			}
			AssetLocations::<T>::insert(currency_id, location.clone());
			LocationToCurrencyIds::<T>::insert(location.clone(), currency_id);
			Self::deposit_event(Event::ForceSetLocation { currency_id, location });
			Ok(().into())
		}

		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn set_existenial_deposit(
			origin: OriginFor<T>,
			currency_id: AssetId,
			existenial_deposit: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let info = DicoAssetsInfo::<T>::get(currency_id).ok_or(Error::<T>::AssetNotExists)?;
			ensure!(who == info.owner, Error::<T>::NotOwner);

			ExistentialDeposits::<T>::insert(currency_id, existenial_deposit);
			Self::deposit_event(Event::SetExistenialDepposit {
				currency_id,
				existenial_deposit,
			});

			Ok(().into())
		}

		/// Users set the asset metadata.
		///
		/// You should have created the asset first.
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn set_metadata(
			origin: OriginFor<T>,
			currency_id: AssetId,
			metadata: DicoAssetMetadata,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			ensure!(
				metadata.name.len() > 2
					&& metadata.symbol.len() > 1
					&& metadata.decimals > 0u8
					&& metadata.decimals < 19,
				Error::<T>::MetadataErr
			);

			let mut asset_info = DicoAssetsInfo::<T>::get(currency_id).ok_or(Error::<T>::AssetNotExists)?;

			ensure!(user == asset_info.owner, Error::<T>::NotOwner);

			match asset_info.metadata.as_ref() {
				Some(x) => {
					ensure!(x != &metadata, Error::<T>::MetadataNotChange);
					ensure!(x.decimals == metadata.decimals, Error::<T>::ShouldNotChangeDecimals);
				}
				None => {}
			}

			asset_info.metadata = Some(metadata.clone());

			DicoAssetsInfo::<T>::insert(currency_id, asset_info);
			Self::deposit_event(Event::SetMetadata(user, currency_id, metadata));
			Ok(().into())
		}

		/// call id:901
		///
		/// Users destroy their own assets.
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn burn(origin: OriginFor<T>, currency_id: AssetId, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			ensure!(Self::is_exists_metadata(currency_id), Error::<T>::MetadataNotExists);

			<T as module::Config>::MultiCurrency::withdraw(currency_id, &user, amount)?;
			Self::deposit_event(Event::Burn(user, currency_id, amount));
			Ok(().into())
		}

		/// call id:902
		///
		/// Transfer some balance to another account under `currency_id`.
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: AssetId,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			if let Some(dao_id) = Daos::<T>::get(currency_id) {
				ensure!(
					pallet_vc::Pallet::<T>::is_open_cex_transfer(dao_id),
					Error::<T>::CexTransferClosed
				);
			};

			match Daos::<T>::get(currency_id) {
				Some(dao_id) => {
					let dao_account = dao::Pallet::<T>::try_get_dao_account_id(dao_id)?;
					let fee: Fee<VcBalanceOf<T>, Permill> = pallet_vc::Pallet::<T>::fees(dao_id);
					match fee {
						Fee::Permill(x) => {
							let real_fee = x * amount;
							<T as module::Config>::MultiCurrency::transfer(currency_id, &from, &dao_account, real_fee)?;
							<T as module::Config>::MultiCurrency::reserve(currency_id, &dao_account, real_fee);
						}
						Fee::Amount(x) => {
							<T as pallet_vc::Config>::MultiCurrency::transfer(
								T::USDCurrencyId::get(),
								&from,
								&dao_account,
								x,
							)?;
							<T as pallet_vc::Config>::MultiCurrency::reserve(T::USDCurrencyId::get(), &dao_account, x);
						}
					}
				}
				_ => {}
			};

			ensure!(Self::is_exists_metadata(currency_id), Error::<T>::MetadataNotExists);

			<Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}


		/// update amount of account `who` under `currency_id`.
		///
		/// The dispatch origin of this call must be _Root_.
		#[pallet::weight(KICO_BASE_WEIGHT)]
		pub fn update_balance(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			currency_id: AssetId,
			amount: AmountOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(Self::is_exists_metadata(currency_id), Error::<T>::MetadataNotExists);
			let dest = T::Lookup::lookup(who)?;
			<Self as MultiCurrencyExtended<T::AccountId>>::update_balance(currency_id, &dest, amount)?;
			Ok(().into())
		}
	}
}

impl<T: Config> CurrenciesHandler<AssetId, DicoAssetMetadata, DispatchError, T::AccountId, BalanceOf<T>, DispatchResult>
	for Pallet<T>
{
	fn get_metadata(currency_id: AssetId) -> result::Result<DicoAssetMetadata, DispatchError> {
		let mut asset_info_opt = DicoAssetsInfo::<T>::get(currency_id);
		let asset_info = match asset_info_opt {
			Some(x) => x,
			_ => {
				if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
					return Ok(DicoAssetMetadata {
						name: vec![],
						symbol: vec![],
						decimals: 12,
					});
				} else {
					return Err(Error::<T>::AssetNotExists)?;
				}
			}
		};
		match asset_info.metadata {
			Some(x) => Ok(x),
			None => Err(Error::<T>::MetadataNotExists)?,
		}
	}

	fn do_create(
		user: T::AccountId,
		currency_id: AssetId,
		metadata: Option<DicoAssetMetadata>,
		amount: BalanceOf<T>,
		is_swap_deposit: bool,
	) -> DispatchResult {
		ensure!(
			!Self::is_exists_metadata(currency_id)
				&& <T as module::Config>::MultiCurrency::total_issuance(currency_id) == BalanceOf::<T>::from(0u32),
			Error::<T>::AssetAlreadyExists
		);
		if is_swap_deposit {
			ensure!(
				Self::is_currency_id_too_large(currency_id),
				Error::<T>::CurrencyIdTooLow
			);
		} else {
			ensure!(
				!Self::is_currency_id_too_large(currency_id),
				Error::<T>::CurrencyIdTooLarge
			);
			#[cfg(test)]
			println!(
				"asset_id:{:?}，free amount: {:?}, Consume:{:?}",
				T::GetNativeCurrencyId::get(),
				T::NativeCurrency::free_balance(&user),
				T::CreateConsume::get()
			);

			Self::withdraw(T::GetNativeCurrencyId::get(), &user, T::CreateConsume::get())?;
		}
		<T as module::Config>::MultiCurrency::deposit(currency_id, &user, amount)?;
		DicoAssetsInfo::<T>::insert(
			currency_id,
			DicoAssetInfo {
				owner: user.clone(),
				metadata: metadata,
			},
		);
		Self::deposit_event(Event::CreateAsset(user.clone(), currency_id, amount));

		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	pub fn try_create_dao(who: &T::AccountId, asset_id: AssetId, dao_id: T::DaoId) -> DispatchResult {
		ensure!(Self::is_owner(asset_id, &who), Error::<T>::NotOwner);
		ensure!(!Daos::<T>::contains_key(asset_id), Error::<T>::DaoExists);
		Daos::<T>::insert(asset_id, dao_id);
		Ok(())
	}

	fn is_exists_metadata(currency_id: AssetId) -> bool {
		// if currency_id == T::GetNativeCurrencyId::get() {
		// 	return true;
		// }
		match DicoAssetsInfo::<T>::get(currency_id).as_ref() {
			Some(x) => {
				if x.metadata.is_some() {
					return true;
				}
			}
			None => {}
		}
		false
	}

	pub fn is_owner(currency_id: AssetId, who: &T::AccountId) -> bool {
		if let Some(info) = DicoAssetsInfo::<T>::get(currency_id).as_ref() {
			return &info.owner == who;
		}
		false
	}

	fn is_currency_id_too_large(currency_id: AssetId) -> bool {
		if currency_id >= T::MaxCreatableCurrencyId::get() {
			return true;
		}
		false
	}
}

impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
	type CurrencyId = AssetId;
	type Balance = BalanceOf<T>;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			<T as module::Config>::NativeCurrency::minimum_balance()
		} else {
			<T as module::Config>::MultiCurrency::minimum_balance(currency_id)
		}
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::total_issuance()
		} else {
			<T as module::Config>::MultiCurrency::total_issuance(currency_id)
		}
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::total_balance(who)
		} else {
			<T as module::Config>::MultiCurrency::total_balance(currency_id, who)
		}
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::free_balance(who)
		} else {
			<T as module::Config>::MultiCurrency::free_balance(currency_id, who)
		}
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::ensure_can_withdraw(who, amount)
		} else {
			<T as module::Config>::MultiCurrency::ensure_can_withdraw(currency_id, who, amount)
		}
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		mut amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}

		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::transfer(from, to, amount)?;
		} else {
			<T as module::Config>::MultiCurrency::transfer(currency_id, from, to, amount)?;
		}
		Self::deposit_event(Event::Transferred(currency_id, from.clone(), to.clone(), amount));
		Ok(())
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::deposit(who, amount)?;
		} else {
			<T as module::Config>::MultiCurrency::deposit(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
		Ok(())
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::withdraw(who, amount)?;
		} else {
			<T as module::Config>::MultiCurrency::withdraw(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Withdrawn(currency_id, who.clone(), amount));
		Ok(())
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> bool {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::can_slash(who, amount)
		} else {
			<T as module::Config>::MultiCurrency::can_slash(currency_id, who, amount)
		}
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::slash(who, amount)
		} else {
			<T as module::Config>::MultiCurrency::slash(currency_id, who, amount)
		}
	}
}

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
	type Amount = AmountOf<T>;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::update_balance(who, by_amount)?;
		} else {
			<T as module::Config>::MultiCurrency::update_balance(currency_id, who, by_amount)?;
		}
		Self::deposit_event(Event::BalanceUpdated(currency_id, who.clone(), by_amount));
		Ok(())
	}
}

impl<T: Config> MultiLockableCurrency<T::AccountId> for Pallet<T> {
	type Moment = T::BlockNumber;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::set_lock(lock_id, who, amount)
		} else {
			<T as module::Config>::MultiCurrency::set_lock(lock_id, currency_id, who, amount)
		}
	}

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::extend_lock(lock_id, who, amount)
		} else {
			<T as module::Config>::MultiCurrency::extend_lock(lock_id, currency_id, who, amount)
		}
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::remove_lock(lock_id, who)
		} else {
			<T as module::Config>::MultiCurrency::remove_lock(lock_id, currency_id, who)
		}
	}
}

impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::can_reserve(who, value)
		} else {
			<T as module::Config>::MultiCurrency::can_reserve(currency_id, who, value)
		}
	}

	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::slash_reserved(who, value)
		} else {
			<T as module::Config>::MultiCurrency::slash_reserved(currency_id, who, value)
		}
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::reserved_balance(who)
		} else {
			<T as module::Config>::MultiCurrency::reserved_balance(currency_id, who)
		}
	}

	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::reserve(who, value)
		} else {
			<T as module::Config>::MultiCurrency::reserve(currency_id, who, value)
		}
	}

	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::unreserve(who, value)
		} else {
			<T as module::Config>::MultiCurrency::unreserve(currency_id, who, value)
		}
	}

	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		if currency_id == T::GetNativeCurrencyId::get() {
			T::NativeCurrency::repatriate_reserved(slashed, beneficiary, value, status)
		} else {
			<T as module::Config>::MultiCurrency::repatriate_reserved(currency_id, slashed, beneficiary, value, status)
		}
	}
}

pub struct Currency<T, GetCurrencyId>(marker::PhantomData<T>, marker::PhantomData<GetCurrencyId>);

impl<T, GetCurrencyId> BasicCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<AssetId>,
{
	type Balance = BalanceOf<T>;

	fn minimum_balance() -> Self::Balance {
		<Pallet<T>>::minimum_balance(GetCurrencyId::get())
	}

	fn total_issuance() -> Self::Balance {
		<Pallet<T>>::total_issuance(GetCurrencyId::get())
	}

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::total_balance(GetCurrencyId::get(), who)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T>>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as MultiCurrency<T::AccountId>>::transfer(GetCurrencyId::get(), from, to, amount)
	}

	fn deposit(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::deposit(GetCurrencyId::get(), who, amount)
	}

	fn withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T>>::withdraw(GetCurrencyId::get(), who, amount)
	}

	fn can_slash(who: &T::AccountId, amount: Self::Balance) -> bool {
		<Pallet<T>>::can_slash(GetCurrencyId::get(), who, amount)
	}

	fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		<Pallet<T>>::slash(GetCurrencyId::get(), who, amount)
	}
}

impl<T, GetCurrencyId> BasicCurrencyExtended<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<AssetId>,
{
	type Amount = AmountOf<T>;

	fn update_balance(who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		<Pallet<T> as MultiCurrencyExtended<T::AccountId>>::update_balance(GetCurrencyId::get(), who, by_amount)
	}
}

impl<T, GetCurrencyId> BasicLockableCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<AssetId>,
{
	type Moment = T::BlockNumber;

	fn set_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as MultiLockableCurrency<T::AccountId>>::set_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn extend_lock(lock_id: LockIdentifier, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<Pallet<T> as MultiLockableCurrency<T::AccountId>>::extend_lock(lock_id, GetCurrencyId::get(), who, amount)
	}

	fn remove_lock(lock_id: LockIdentifier, who: &T::AccountId) -> DispatchResult {
		<Pallet<T> as MultiLockableCurrency<T::AccountId>>::remove_lock(lock_id, GetCurrencyId::get(), who)
	}
}

impl<T, GetCurrencyId> BasicReservableCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<AssetId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::slash_reserved(GetCurrencyId::get(), who, value)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		<Pallet<T> as MultiReservableCurrency<T::AccountId>>::repatriate_reserved(
			GetCurrencyId::get(),
			slashed,
			beneficiary,
			value,
			status,
		)
	}
}

pub type NativeCurrencyOf<T> = Currency<T, <T as Config>::GetNativeCurrencyId>;

/// Adapt other currency traits implementation to `BasicCurrency`.
pub struct BasicCurrencyAdapter<T, Currency, Amount, Moment>(marker::PhantomData<(T, Currency, Amount, Moment)>);

type PalletBalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::Balance;

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletCurrency<AccountId>,
	T: Config,
{
	type Balance = PalletBalanceOf<AccountId, Currency>;

	fn minimum_balance() -> Self::Balance {
		Currency::minimum_balance()
	}

	fn total_issuance() -> Self::Balance {
		Currency::total_issuance()
	}

	fn total_balance(who: &AccountId) -> Self::Balance {
		Currency::total_balance(who)
	}

	fn free_balance(who: &AccountId) -> Self::Balance {
		Currency::free_balance(who)
	}

	fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let new_balance = Self::free_balance(who)
			.checked_sub(&amount)
			.ok_or(Error::<T>::BalanceTooLow)?;

		Currency::ensure_can_withdraw(who, amount, WithdrawReasons::all(), new_balance)
	}

	fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)
	}

	fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		let _ = Currency::deposit_creating(who, amount);
		Ok(())
	}

	fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::withdraw(who, amount, WithdrawReasons::all(), ExistenceRequirement::AllowDeath).map(|_| ())
	}

	fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
		Currency::can_slash(who, amount)
	}

	fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash(who, amount);
		gap
	}
}

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrencyExtended<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Amount: Signed
		+ TryInto<PalletBalanceOf<AccountId, Currency>>
		+ TryFrom<PalletBalanceOf<AccountId, Currency>>
		+ SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ MaxEncodedLen
		+ Debug
		+ Default,
	Currency: PalletCurrency<AccountId>,
	T: Config,
{
	type Amount = Amount;

	fn update_balance(who: &AccountId, by_amount: Self::Amount) -> DispatchResult {
		let by_balance = by_amount
			.abs()
			.try_into()
			.map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(who, by_balance)
		} else {
			Self::withdraw(who, by_balance)
		}
	}
}

// Adapt `frame_support::traits::LockableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicLockableCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletLockableCurrency<AccountId>,
	T: Config,
{
	type Moment = Moment;

	fn set_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::set_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn extend_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
		Currency::extend_lock(lock_id, who, amount, WithdrawReasons::all());
		Ok(())
	}

	fn remove_lock(lock_id: LockIdentifier, who: &AccountId) -> DispatchResult {
		Currency::remove_lock(lock_id, who);
		Ok(())
	}
}

// Adapt `frame_support::traits::ReservableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicReservableCurrency<AccountId>
	for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
	Currency: PalletReservableCurrency<AccountId>,
	T: Config,
{
	fn can_reserve(who: &AccountId, value: Self::Balance) -> bool {
		Currency::can_reserve(who, value)
	}

	fn slash_reserved(who: &AccountId, value: Self::Balance) -> Self::Balance {
		let (_, gap) = Currency::slash_reserved(who, value);
		gap
	}

	fn reserved_balance(who: &AccountId) -> Self::Balance {
		Currency::reserved_balance(who)
	}

	fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult {
		Currency::reserve(who, value)
	}

	fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance {
		Currency::unreserve(who, value)
	}

	fn repatriate_reserved(
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		Currency::repatriate_reserved(slashed, beneficiary, value, status)
	}
}

pub struct AssetIdMaps<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> AssetIdMapping<AssetId, MultiLocation> for AssetIdMaps<T> {
	fn get_multi_location(asset_id: AssetId) -> Option<MultiLocation> {
		Pallet::<T>::asset_locations(asset_id)
	}

	fn get_currency_id(multi_location: MultiLocation) -> Option<AssetId> {
		Pallet::<T>::location_to_currency_ids(multi_location)
	}

	fn get_weight_rate_multiple(location: MultiLocation) -> Option<u128> {
		if let Some(id) = Self::get_currency_id(location.clone()) {
			Some(WeightRateMultiple::<T>::get(id))
		} else {
			None
		}
	}
}
