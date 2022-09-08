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

#![allow(unused_imports)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
pub use daos_create_dao::{self as dao, AccountIdConversion, Vec};
use daos_primitives::traits::SetCollectiveMembers;
use dico_primitives::AssetId;
use frame_support::dispatch::UnfilteredDispatchable;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use scale_info::prelude::boxed::Box;
use scale_info::TypeInfo;
use sp_runtime::{Permill, RuntimeDebug};
use sp_std::result::Result;
// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo)]
pub enum Fee<Amount, Permill> {
	Permill(Permill),
	Amount(Amount),
}

impl<Amount: Default> Default for Fee<Amount, Permill> {
	fn default() -> Self {
		Fee::Amount(Amount::default())
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use daos_create_dao::BaseCallFilter;
	use frame_support::traits::Contains;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	type BalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + dao::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type SetCollectiveMembers: SetCollectiveMembers<Self::AccountId, Self::DaoId, DispatchError>;

		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId>
			+ MultiReservableCurrency<Self::AccountId>
			+ MultiCurrencyExtended<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn guarders)]
	pub type Guarders<T: Config> = StorageMap<_, Identity, T::DaoId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fees)]
	pub type Fees<T: Config> = StorageMap<_, Identity, T::DaoId, Fee<BalanceOf<T>, Permill>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_open_cex_transfer)]
	pub type IsOpenCexTransfer<T: Config> = StorageMap<_, Identity, T::DaoId, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		SetGuarders(T::DaoId),
		RemoveGuard(T::DaoId, T::AccountId),
		AddGuarder(T::DaoId, T::AccountId),
		Unreserved(T::DaoId, AssetId, BalanceOf<T>),
		IcoOperate {
			dao_id: T::DaoId,
			call: <T as dao::Config>::Call,
			result: DispatchResult,
		},
		BurnOperate {
			dao_id: T::DaoId,
			call: <T as dao::Config>::Call,
			result: DispatchResult,
		},
		SetFee(T::DaoId, Fee<BalanceOf<T>, Permill>),
		OpenCexTransfer(T::DaoId, bool),
		CloseCexTransfer(T::DaoId, bool),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		BadOrigin,
		HaveNoGurarder,
		GuarderIsExists,
		NotSupprtCall,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// (daos support. call name: set_guarders, call id:701)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_guarders(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			mut guarders: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			ensure!(guarders.len() > 0, Error::<T>::HaveNoGurarder);
			guarders.sort();
			Guarders::<T>::insert(dao_id, guarders);
			T::SetCollectiveMembers::set_members_sorted(dao_id, &Self::guarders(dao_id), None)?;

			Self::deposit_event(Event::<T>::SetGuarders(dao_id));
			Ok(().into())
		}

		/// (daos support. call name: remove_guarder, call id:702)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_guarder(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			guarder: T::AccountId,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			Guarders::<T>::mutate(dao_id, |h| {
				if let Some(pos) = h.iter().position(|k| k == &guarder) {
					h.remove(pos);
				}
			});
			T::SetCollectiveMembers::set_members_sorted(dao_id, &Self::guarders(dao_id), None)?;

			Self::deposit_event(Event::<T>::RemoveGuard(dao_id, guarder));
			Ok(().into())
		}

		/// (daos support. call name: add_guarder, call id:703)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_guarder(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			guarder: T::AccountId,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			let mut guarders = Guarders::<T>::get(dao_id);
			ensure!(!guarders.contains(&guarder), Error::<T>::GuarderIsExists);
			guarders.push(guarder.clone());
			Guarders::<T>::insert(dao_id, guarders);
			T::SetCollectiveMembers::set_members_sorted(dao_id, &Self::guarders(dao_id), None)?;

			Self::deposit_event(Event::<T>::AddGuarder(dao_id, guarder));
			Ok(().into())
		}

		/// (daos support. call name: unreserve, call id:704)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn unreserve(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			asset_id: AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			T::MultiCurrency::unreserve(asset_id, &who, amount);
			Self::deposit_event(Event::<T>::Unreserved(dao_id, asset_id, amount));
			Ok(().into())
		}

		/// (daos support. call name: set_fee, call id:705)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_fee(
			origin: OriginFor<T>,
			dao_id: T::DaoId,
			fee: Fee<BalanceOf<T>, Permill>,
		) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			Fees::<T>::insert(dao_id, &fee);
			Self::deposit_event(Event::<T>::SetFee(dao_id, fee));
			Ok(().into())
		}

		/// (daos support. call name: open_cex_transfer, call id:706)
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn open_cex_transfer(origin: OriginFor<T>, dao_id: T::DaoId) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			IsOpenCexTransfer::<T>::insert(dao_id, true);
			Self::deposit_event(Event::<T>::OpenCexTransfer(dao_id, true));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn close_cex_transfer(origin: OriginFor<T>, dao_id: T::DaoId) -> DispatchResultWithPostInfo {
			dao::Pallet::<T>::ensrue_dao_root(origin, dao_id)?;
			IsOpenCexTransfer::<T>::insert(dao_id, false);
			Self::deposit_event(Event::<T>::CloseCexTransfer(dao_id, false));
			Ok(().into())
		}


	}
}
