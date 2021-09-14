// This file is part of DICO.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

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

// NOTICE
// This module is changed from the treasury module in Substrate.
// You can find the Treasury at
//
// https://github.com/paritytech/substrate/tree/master/frame/treasury
//

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod tests;
pub mod traits;
pub mod weights;

use codec::{Decode, Encode};
use frame_support::traits::EnsureOrigin;
use frame_support::traits::{
	Currency, ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced, ReservableCurrency, WithdrawReasons,
};
use frame_support::weights::{DispatchClass, Weight};
use frame_support::{PalletId, decl_error, decl_event, decl_module, decl_storage, ensure, print, StorageDoubleMap};
use frame_system::ensure_signed;
use traits::DicoTreasuryHandler;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::SaturatedConversion;
use sp_runtime::{
	traits::{AccountIdConversion, Saturating, StaticLookup, Zero},
	Permill, RuntimeDebug,
};
use sp_std::prelude::*;
pub use weights::WeightInfo;

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

pub(crate) type BalanceOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

pub trait Config: frame_system::Config {
	/// Origin from which approvals must come.
	type ApproveOrigin: EnsureOrigin<Self::Origin>;

	type PalletId: Get<PalletId>;

	type MultiCurrency: MultiCurrency<Self::AccountId>
		+ MultiCurrencyExtended<Self::AccountId>
		+ MultiLockableCurrency<Self::AccountId>
		+ MultiReservableCurrency<Self::AccountId>;

	/// Origin from which rejections must come.
	type RejectOrigin: EnsureOrigin<Self::Origin>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

	/// Minimum amount of funds that should be placed in a deposit for making a
	/// proposal.
	type ProposalBond: Get<BalanceOf<Self>>;

	/// Period between successive spends.
	type SpendPeriod: Get<Self::BlockNumber>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

// /// An index of a proposal. Just a `u32`.
pub type ProposalIndex = u32;

/// A spending proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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

decl_storage! {
	trait Store for Module<T: Config> as Treasury {
		/// Number of proposals that have been made.
		ProposalCount get(fn proposal_count): ProposalIndex;

		/// Proposals that have been made.
		pub Proposals get(fn proposals):
			map hasher(identity) ProposalIndex
			=> Option<DicoTreasuryProposal<T::AccountId, BalanceOf<T>, T::BlockNumber, CurrencyIdOf<T>>>;

		/// Proposal indices that have been approved but not yet awarded.
		pub Approvals get(fn approvals): Vec<ProposalIndex>;
	}

}

decl_event!(
	pub enum Event<T>
	where
		Balance = BalanceOf<T>,
		<T as frame_system::Config>::AccountId,
	{
		/// New proposal. \[proposal_index\]
		Proposed(ProposalIndex),
		/// We have ended a spend period and will now allocate funds. \[budget_remaining\]
		Spending(Balance),
		/// Some funds have been allocated. \[proposal_index, award, beneficiary\]
		Awarded(ProposalIndex, Balance, AccountId),
		/// A proposal was rejected; funds were slashed. \[proposal_index, slashed\]
		Rejected(ProposalIndex, Balance),
		/// A proposal was approved;
		Approved(ProposalIndex),
		/// Some of our funds have been burnt. \[burn\]
		Burnt(Balance),
		/// Spending has finished; this is the amount that rolls over until next spend.
		/// \[budget_remaining\]
		Rollover(Balance),
		/// Some funds have been deposited. \[deposit\]
		Deposit(Balance),
		SpendFund,
	}
);

decl_error! {
	/// Error for the treasury module.
	pub enum Error for Module<T: Config> {
		/// Proposer's balance is too low.
		InsufficientProposersBalance,
		/// No proposal or bounty at that index.
		InvalidIndex,
		ApprovalsIsempty,
	}
}
//
decl_module! {
	pub struct Module<T: Config>
		for enum Call
		where origin: T::Origin
	{
		/// Fraction of a proposal's value that should be bonded in order to place the proposal.
		/// An accepted proposal gets these back. A rejected proposal does not.
		const ProposalBond: BalanceOf<T> = T::ProposalBond::get();

		const SpendPeriod: T::BlockNumber = T::SpendPeriod::get();

		const PalletId: PalletId = T::PalletId::get();

		type Error = Error<T>;

		fn deposit_event() = default;


		/// The user makes a proposal about funding
		#[weight = 10000 + T::DbWeight::get().reads_writes(3, 3)]
		pub fn propose_spend(
			origin,
			currency_id: CurrencyIdOf<T>,
			#[compact] value: BalanceOf<T>,
			beneficiary: <T::Lookup as StaticLookup>::Source
		) {
			let proposer = ensure_signed(origin)?;

			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let bond = T::ProposalBond::get();

			T::MultiCurrency::reserve(T::GetNativeCurrencyId::get(), &proposer, bond)
				.map_err(|_| Error::<T>::InsufficientProposersBalance)?;

			let c = Self::proposal_count();
			let start_spend_time = None;
			<ProposalCount>::put(c + 1);
			<Proposals<T>>::insert(c, DicoTreasuryProposal { currency_id, proposer, value, beneficiary, bond, start_spend_time });

			Self::deposit_event(RawEvent::Proposed(c));
		}


		/// The council rejected the proposal.
		#[weight = 10000 + T::DbWeight::get().reads_writes(1, 2)]
		pub fn reject_proposal(origin, #[compact] proposal_id: ProposalIndex) {
			T::RejectOrigin::ensure_origin(origin)?;

			let proposal = <Proposals<T>>::take(&proposal_id).ok_or(Error::<T>::InvalidIndex)?;
			let value = proposal.bond;
			let imbalance = T::MultiCurrency::slash_reserved(T::GetNativeCurrencyId::get(), &proposal.proposer, value);

			Self::deposit_event(Event::<T>::Rejected(proposal_id, value));
		}


		/// The council approve the proposal.
		#[weight = 10000 + T::DbWeight::get().reads_writes(1, 2)]
		pub fn approve_proposal(origin, #[compact] proposal_id: ProposalIndex) {
			T::ApproveOrigin::ensure_origin(origin)?;

			ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::InvalidIndex);

			<Proposals<T>>::mutate(proposal_id, |h| if let Some(p) = h {
				p.start_spend_time = Some(<frame_system::Module<T>>::block_number() + T::SpendPeriod::get());
			});
			<Approvals>::mutate(|h| h.push(proposal_id));

			Self::deposit_event(Event::<T>::Approved(proposal_id));
		}


		/// Users get their fund.
		#[weight = 10000 + T::DbWeight::get().reads_writes(4, 4)]
		pub fn spend_fund(origin) {
			let who = ensure_signed(origin)?;
			let mut proposal_ids = <Approvals>::get();
			if proposal_ids.len() == 0 {
				return Err(Error::<T>::ApprovalsIsempty)?;
			}

			for proposal_id in proposal_ids.clone().iter() {
				if let Some(proposal) =  <Proposals<T>>::get(proposal_id) {
					if proposal.start_spend_time.is_some() && proposal.start_spend_time.unwrap() <= <frame_system::Module<T>>::block_number() {

						let transfer_result = T::MultiCurrency::transfer(proposal.currency_id, &Self::get_treasury_account_id(), &proposal.beneficiary, proposal.value);
						if transfer_result.is_ok() {
							T::MultiCurrency::unreserve(T::GetNativeCurrencyId::get(), &proposal.proposer, proposal.bond);
							proposal_ids.retain(|h| h != proposal_id);
							<Proposals<T>>::remove(proposal_id);
						}
					}
				}
			}
			<Approvals>::put(proposal_ids);

			Self::deposit_event(Event::<T>::SpendFund);
		}

	}
}

impl<T: Config> DicoTreasuryHandler<T::AccountId> for Module<T> {
	fn get_treasury_account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}
