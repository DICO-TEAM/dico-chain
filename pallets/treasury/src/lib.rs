// Forked from https://github.com/paritytech/substrate/tree/master/frame/treasury

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

mod benchmarking;
pub mod traits;
pub mod weights;
mod tests;

pub use crate::pallet::*;

use codec::{Decode, Encode};
use frame_support::traits::EnsureOrigin;
use frame_support::traits::{
	Currency, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced, ReservableCurrency, WithdrawReasons,
};
use frame_support::weights::{DispatchClass, Weight};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, print, PalletId, StorageDoubleMap};
use frame_system::ensure_signed;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_runtime::{
	traits::{AccountIdConversion, Saturating, StaticLookup, Zero},
	Permill, RuntimeDebug,
};
use sp_std::prelude::*;
use traits::DicoTreasuryHandler;
pub use weights::WeightInfo;
use scale_info::TypeInfo;
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

pub type ProposalIndex = u32;

pub(crate) type BalanceOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

/// A spending proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct DicoTreasuryProposal<AccountId, Balance, BlockNumber, CurrencyId> {
	currency_id: CurrencyId,
	/// The account proposing it.
	proposer: AccountId,
	/// The (total) amount that should be paid if the proposal is accepted.
	value: Balance,
	/// The account to whom the payment should be made if the proposal is
	/// accepted.
	beneficiary: AccountId,
	/// The amount held on deposit (reserved) for making this proposal.
	bond: Balance,
	start_spend_time: Option<BlockNumber>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Origin from which approvals must come.
		type ApproveOrigin: EnsureOrigin<Self::Origin>;
		type MultiCurrency: MultiCurrency<Self::AccountId>
			+ MultiCurrencyExtended<Self::AccountId>
			+ MultiLockableCurrency<Self::AccountId>
			+ MultiReservableCurrency<Self::AccountId>;
		/// Origin from which rejections must come.
		type RejectOrigin: EnsureOrigin<Self::Origin>;
		/// The overarching event type.
		type Event: From<Event<Self>>
			+ Into<<Self as frame_system::Config>::Event>
			+ IsType<<Self as frame_system::Config>::Event>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;
		/// Minimum amount of funds that should be placed in a deposit for making a
		/// proposal.
		#[pallet::constant]
		type ProposalBond: Get<BalanceOf<Self>>;
		/// Period between successive spends.
		type SpendPeriod: Get<Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalIndex,
		DicoTreasuryProposal<T::AccountId, BalanceOf<T>, T::BlockNumber, CurrencyIdOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub type ProposalCount<T: Config> = StorageValue<_, ProposalIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type Approvals<T: Config> = StorageValue<_, Vec<ProposalIndex>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The user makes a proposal about funding
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(3, 3))]
		pub fn propose_spend(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			#[pallet::compact] value: BalanceOf<T>,
			beneficiary: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let bond = Self::calculate_bond(value);

			T::MultiCurrency::reserve(T::GetNativeCurrencyId::get(), &proposer, bond)
				.map_err(|_| Error::<T>::InsufficientProposersBalance)?;

			let c = Self::proposal_count();
			let start_spend_time = None;
			<ProposalCount<T>>::put(c + 1);
			<Proposals<T>>::insert(
				c,
				DicoTreasuryProposal {
					currency_id,
					proposer,
					value,
					beneficiary,
					bond,
					start_spend_time,
				},
			);

			Self::deposit_event(Event::Proposed(c));
			Ok(())
		}

		/// The council rejected the proposal.
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn reject_proposal(origin: OriginFor<T>, #[pallet::compact] proposal_id: ProposalIndex) -> DispatchResult {
			T::RejectOrigin::ensure_origin(origin)?;

			let proposal = <Proposals<T>>::take(&proposal_id).ok_or(Error::<T>::InvalidIndex)?;
			let value = proposal.bond;
			let imbalance = T::MultiCurrency::slash_reserved(T::GetNativeCurrencyId::get(), &proposal.proposer, value);

			Self::deposit_event(Event::<T>::Rejected(proposal_id, value));
			Ok(())
		}

		/// The council approve the proposal.
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn approve_proposal(origin: OriginFor<T>, #[pallet::compact] proposal_id: ProposalIndex) -> DispatchResult {
			T::ApproveOrigin::ensure_origin(origin)?;

			ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::InvalidIndex);

			<Proposals<T>>::mutate(proposal_id, |h| {
				if let Some(p) = h {
					p.start_spend_time = Some(<frame_system::Module<T>>::block_number() + T::SpendPeriod::get());
				}
			});
			<Approvals<T>>::mutate(|h| h.push(proposal_id));

			Self::deposit_event(Event::<T>::Approved(proposal_id));
			Ok(())
		}

		/// Users get their fund.
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(4, 4))]
		pub fn spend_fund(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut proposal_ids = <Approvals<T>>::get();
			if proposal_ids.len() == 0 {
				return Err(Error::<T>::ApprovalsIsempty)?;
			}

			for proposal_id in proposal_ids.clone().iter() {
				if let Some(proposal) = <Proposals<T>>::get(proposal_id) {
					if proposal.start_spend_time.is_some()
						&& proposal.start_spend_time.unwrap() <= <frame_system::Module<T>>::block_number()
					{
						let transfer_result = T::MultiCurrency::transfer(
							proposal.currency_id,
							&Self::get_treasury_account_id(),
							&proposal.beneficiary,
							proposal.value,
						);
						if transfer_result.is_ok() {
							T::MultiCurrency::unreserve(
								T::GetNativeCurrencyId::get(),
								&proposal.proposer,
								proposal.bond,
							);
							proposal_ids.retain(|h| h != proposal_id);
							<Proposals<T>>::remove(proposal_id);
						}
					}
				}
			}
			<Approvals<T>>::put(proposal_ids);

			Self::deposit_event(Event::<T>::SpendFund);
			Ok(())
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Proposer's balance is too low.
		InsufficientProposersBalance,
		/// No proposal or bounty at that index.
		InvalidIndex,
		ApprovalsIsempty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New proposal. \[proposal_index\]
		Proposed(ProposalIndex),
		/// We have ended a spend period and will now allocate funds. \[budget_remaining\]
		Spending(BalanceOf<T>),
		/// Some funds have been allocated. \[proposal_index, award, beneficiary\]
		Awarded(ProposalIndex, BalanceOf<T>, T::AccountId),
		/// A proposal was rejected; funds were slashed. \[proposal_index, slashed\]
		Rejected(ProposalIndex, BalanceOf<T>),
		/// A proposal was approved;
		Approved(ProposalIndex),
		/// Some of our funds have been burnt. \[burn\]
		Burnt(BalanceOf<T>),
		/// Spending has finished; this is the amount that rolls over until next spend.
		/// \[budget_remaining\]
		Rollover(BalanceOf<T>),
		/// Some funds have been deposited. \[deposit\]
		Deposit(BalanceOf<T>),
		SpendFund,
	}
}

impl <T: Config> Pallet<T> {
	pub fn calculate_bond(value: BalanceOf<T>) -> BalanceOf<T> {
			T::ProposalBond::get() * value
		}

}

impl<T: Config> DicoTreasuryHandler<T::AccountId> for Module<T> {
	fn get_treasury_account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}
