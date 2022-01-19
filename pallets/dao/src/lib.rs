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
#![recursion_limit = "128"]

// use codec::{Codec, Encode, Decode, MaxEncodedLen};
use frame_support::{
	codec::{Codec, Decode, Encode},
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo, Dispatchable, Parameter, PostDispatchInfo},
	ensure,
	traits::{ChangeMembers, Currency, EnsureOrigin, Get, InitializeMembers, ReservableCurrency},
	weights::{DispatchClass, GetDispatchInfo, Pays, Weight},
};
use scale_info::TypeInfo;
use frame_system::{self as system, ensure_root, ensure_signed};
use ico;
use ico::traits::IcoHandler;
use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
use pallet_timestamp;
use sp_core::u32_trait::Value as U32;
use sp_io;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member, SaturatedConversion, Saturating};
use sp_runtime::{traits::Hash, Percent, RuntimeDebug};
use sp_std::convert::From;
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	prelude::*,
	result,
};
pub use weights::WeightInfo;
pub use crate::pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
//
pub mod weights;
pub mod mock;
#[cfg(test)]
pub mod tests;


/// Simple index type for proposal counting.
pub type ProposalIndex = u32;
pub type RoomIndex = u64;

pub(crate) type MultiBalanceOf<T> =
	<<T as ico::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type CurrencyIdOf<T> =
	<<T as ico::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

/// A number of members.
///
/// This also serves as a number of voting members, and since for motions, each
/// member may vote exactly once, therefore also the number of votes for any
/// given motion.
pub type MemberCount = u32;

/// Origin for the collective module.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum IcoRawOrigin<AccountId, MulBalance> {
	/// It has been condoned by a given number of members of the collective from
	/// a given total.
	Members(MulBalance, MulBalance),
	/// It has been condoned by a single member of the collective.
	Member(MulBalance, AccountId),
	_Phantom(sp_std::marker::PhantomData<(AccountId, MulBalance)>),
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Info for keeping track of a motion being voted on.
pub struct IcoCollectiveVotes<AccountId, BlockNumber, MulBalance> {
	/// The proposal's unique index.
	pub index: ProposalIndex,
	/// The proposal's reason,
	pub reason: Vec<u8>,
	/// The number of approval IcoCollectiveVotes that are needed to pass the
	/// motion.
	pub threshold: Percent,
	/// The current set of voters that approved it.
	pub ayes: Vec<(AccountId, MulBalance)>,
	/// The current set of voters that rejected it.
	pub nays: Vec<(AccountId, MulBalance)>,
	/// The hard end time of this vote.
	pub end: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + ico::Config {
		/// The outer origin type.
		type Origin: From<IcoRawOrigin<Self::AccountId, MultiBalanceOf<Self>>>
			+ Into<Result<IcoRawOrigin<Self::AccountId, MultiBalanceOf<Self>>, <Self as Config>::Origin>>;
		/// The outer call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<Origin = <Self as Config>::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ GetDispatchInfo;
		/// The outer event type.
		type Event: From<Event<Self>>
			+ Into<<Self as frame_system::Config>::Event>
			+ IsType<<Self as frame_system::Config>::Event>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		type IcoHandler: IcoHandler<
			CurrencyIdOf<Self>,
			MultiBalanceOf<Self>,
			Self::AccountId,
			DispatchError,
			Self::BlockNumber,
		>;
		/// The time-out for council motions.
		#[pallet::constant]
		type MotionDuration: Get<Self::BlockNumber>;
		/// Maximum number of proposals allowed to be active in parallel.
		#[pallet::constant]
		type MaxProposals: Get<ProposalIndex>;
	}

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyIdOf<T>, Vec<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, CurrencyIdOf<T>, Blake2_128Concat, T::Hash, T::Proposal>;

	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type Voting<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CurrencyIdOf<T>,
		Blake2_128Concat,
		T::Hash,
		IcoCollectiveVotes<T::AccountId, T::BlockNumber, MultiBalanceOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub type ProposalCount<T: Config> = StorageMap<_, Blake2_128Concat, CurrencyIdOf<T>, u32, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The user makes a proposal.
		///
		/// Must be a member of the project ICO.
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(10, 5))]
		pub fn propose(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			#[pallet::compact] threshold: Percent,
			proposal: Box<<T as Config>::Proposal>,
			reason: Vec<u8>,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::IcoHandler::is_project_ico_member(currency_id, ico_index, &who)?,
				Error::<T>::NotIcoMember
			);
			let user_ico_amount = T::IcoHandler::get_user_total_amount(currency_id, ico_index, &who);
			let ico_total_amount = T::IcoHandler::get_project_total_ico_amount(currency_id, ico_index)?;

			let proposal_len = proposal.using_encoded(|x| x.len());
			// ensure!(proposal_len <= length_bound as usize, Error::<T>::WrongProposalLength);
			let proposal_hash = T::Hashing::hash_of(&proposal);
			ensure!(
				!<ProposalOf<T>>::contains_key(currency_id, proposal_hash),
				Error::<T>::DuplicateProposal
			);

			if threshold * ico_total_amount <= user_ico_amount {
				let result = proposal.dispatch(IcoRawOrigin::Members(user_ico_amount, ico_total_amount).into());
				Self::deposit_event(Event::Executed(proposal_hash, result.map(|_| ()).map_err(|e| e.error)));
			} else {
				let mut proposals = Proposals::<T>::get(currency_id);
				ensure!(
					proposals.len() + 1 <= T::MaxProposals::get() as usize,
					Error::<T>::TooManyProposals
				);
				proposals.push(proposal_hash);
				Proposals::<T>::insert(currency_id, proposals);

				let index = Self::proposal_count(currency_id);
				<ProposalCount<T>>::mutate(currency_id, |i| *i += 1);
				<ProposalOf<T>>::insert(currency_id, proposal_hash, *proposal);
				let end = system::Pallet::<T>::block_number() + T::MotionDuration::get();
				let votes = IcoCollectiveVotes {
					index,
					reason: reason,
					threshold,
					ayes: vec![(who.clone(), user_ico_amount)],
					nays: vec![],
					end,
				};
				<Voting<T>>::insert(currency_id, proposal_hash, votes);

				Self::deposit_event(Event::Proposed(who, index, proposal_hash, threshold));
			}
			Ok(())
		}

		/// Users vote on proposals.
		///
		/// Must be a member of the project ICO
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(10, 2))]
		pub fn vote(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			proposal: T::Hash,
			#[pallet::compact] index: ProposalIndex,
			approve: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				T::IcoHandler::is_project_ico_member(currency_id, ico_index, &who)?,
				Error::<T>::NotIcoMember
			);
			let user_ico_amount = T::IcoHandler::get_user_total_amount(currency_id, ico_index, &who);
			let ico_total_amount = T::IcoHandler::get_project_total_ico_amount(currency_id, ico_index)?;

			let mut voting = Self::voting(currency_id, &proposal).ok_or(Error::<T>::ProposalMissing)?;

			ensure!(voting.index == index, Error::<T>::WrongIndex);

			ensure!(
				system::Pallet::<T>::block_number() <= voting.end,
				Error::<T>::VoteExpire
			);

			let position_yes = voting.ayes.iter().position(|a| a.0 == who.clone());
			let position_no = voting.nays.iter().position(|a| a.0 == who.clone());

			// Detects first vote of the member in the motion
			let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

			if approve {
				if position_yes.is_none() {
					voting.ayes.push((who.clone(), user_ico_amount));
				} else {
					let this_vote = voting.ayes.swap_remove(position_yes.unwrap());
					let vote_amount = user_ico_amount.saturating_sub(this_vote.1);
					ensure!(vote_amount > MultiBalanceOf::<T>::from(0u32), Error::<T>::DuplicateVote);
					voting.ayes.push((this_vote.0, user_ico_amount));
				}
				if let Some(pos) = position_no {
					voting.nays.swap_remove(pos);
				}
			} else {
				if position_no.is_none() {
					voting.nays.push((who.clone(), user_ico_amount));
				} else {
					let mut this_vote = voting.nays.swap_remove(position_no.unwrap());
					let vote_amount = user_ico_amount.saturating_sub(this_vote.1);
					ensure!(vote_amount > MultiBalanceOf::<T>::from(0u32), Error::<T>::DuplicateVote);
					voting.nays.push((this_vote.0, user_ico_amount));
				}
				if let Some(pos) = position_yes {
					voting.ayes.swap_remove(pos);
				}
			}

			let mut yes_votes = MultiBalanceOf::<T>::from(0u32);
			let mut no_votes = MultiBalanceOf::<T>::from(0u32);
			for i in voting.ayes.iter() {
				yes_votes = yes_votes.saturating_add(i.1);
			}
			for i in voting.nays.iter() {
				no_votes = no_votes.saturating_add(i.1);
			}

			let seats = ico_total_amount;

			Voting::<T>::insert(currency_id, &proposal, voting.clone());

			Self::deposit_event(Event::Voted(who, proposal, approve, seats, yes_votes, no_votes));

			Ok(())
		}

		/// The user close the proposal.
		///
		/// Everyone can do it
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(10, 3))]
		pub fn close(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			ico_index: u32,
			proposal_hash: T::Hash,
			#[pallet::compact] index: ProposalIndex,
			#[pallet::compact] proposal_weight_bound: Weight,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let voting = Self::voting(currency_id, &proposal_hash).ok_or(Error::<T>::ProposalMissing)?;

			ensure!(voting.index == index, Error::<T>::WrongIndex);

			let mut yes_votes = MultiBalanceOf::<T>::from(0u32);
			let mut no_votes = MultiBalanceOf::<T>::from(0u32);
			for i in voting.ayes.iter() {
				yes_votes = yes_votes.saturating_add(i.1);
			}
			for i in voting.nays.iter() {
				no_votes = no_votes.saturating_add(i.1);
			}

			let seats = T::IcoHandler::get_project_total_ico_amount(currency_id, ico_index)?;

			let approved = yes_votes >= voting.threshold * seats;
			let mut disapproved = seats.saturating_sub(no_votes) <= voting.threshold * seats;
			if approved && disapproved {
				disapproved = false;
			}

			// Allow (dis-)approving the proposal as soon as there are enough votes.
			if approved {
				let proposal = ProposalOf::<T>::get(currency_id, proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
				Self::deposit_event(Event::Closed(proposal_hash, yes_votes, no_votes));
				let (proposal_weight, proposal_count) =
					Self::do_approve_proposal(currency_id, seats, yes_votes, proposal_hash, proposal);
				return Ok(());
			} else if disapproved {
				Self::deposit_event(Event::Closed(proposal_hash, yes_votes, no_votes));
				let proposal_count = Self::do_disapprove_proposal(currency_id, proposal_hash);
				return Ok(());
			}

			// Only allow actual closing of the proposal after the voting period has ended.
			ensure!(system::Pallet::<T>::block_number() >= voting.end, Error::<T>::TooEarly);
			Self::deposit_event(Event::Closed(proposal_hash, yes_votes, no_votes));
			let proposal_count = Self::do_disapprove_proposal(currency_id, proposal_hash);

			Ok(())
		}

		/// The root user disapprove the proposal.
		///
		/// Referendum
		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(0, 5))]
		pub fn disapprove_proposal(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			proposal_hash: T::Hash,
		) -> DispatchResult {
			ensure_root(origin)?;
			let proposal_count = Self::do_disapprove_proposal(currency_id, proposal_hash);
			Ok(())
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Duplicate proposals not allowed
		DuplicateProposal,
		/// Proposal must exist
		ProposalMissing,
		/// Mismatched index
		WrongIndex,
		NotIcoMember,
		/// Duplicate vote ignored
		DuplicateVote,
		/// Members are already initialized!
		AlreadyInitialized,
		/// The close call was made too early, before the end of the voting.
		TooEarly,
		/// There can only be a maximum of `MaxProposals` active proposals.
		TooManyProposals,
		/// The given weight bound for the proposal was too low.
		WrongProposalWeight,
		/// The given length bound for the proposal was too low.
		WrongProposalLength,
		VoteExpire,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		/// \[account, proposal_index, proposal_hash, threshold\]
		Proposed(T::AccountId, ProposalIndex, T::Hash, Percent),
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		/// \[account, proposal_hash, voted, yes, no\]
		Voted(
			T::AccountId,
			T::Hash,
			bool,
			MultiBalanceOf<T>,
			MultiBalanceOf<T>,
			MultiBalanceOf<T>,
		),
		/// A motion was approved by the required threshold.
		/// \[proposal_hash\]
		Approved(T::Hash),
		/// A motion was not approved by the required threshold.
		/// \[proposal_hash\]
		Disapproved(T::Hash),
		/// A motion was executed; result will be `Ok` if it returned without error.
		/// \[proposal_hash, result\]
		Executed(T::Hash, DispatchResult),
		/// A single member did some action; result will be `Ok` if it returned without error.
		/// \[proposal_hash, result\]
		MemberExecuted(T::Hash, DispatchResult),
		/// A proposal was closed because its threshold was reached or after its duration was up.
		/// \[proposal_hash, yes, no\]
		Closed(T::Hash, MultiBalanceOf<T>, MultiBalanceOf<T>),
	}

	impl<T: Config> Pallet<T> {
		fn validate_and_get_proposal(
			currency_id: CurrencyIdOf<T>,
			hash: &T::Hash,
			length_bound: u32,
			weight_bound: Weight,
		) -> Result<(<T as Config>::Proposal, usize), DispatchError> {
			let key = ProposalOf::<T>::hashed_key_for(currency_id, hash);
			// read the length of the proposal storage entry directly
			let proposal_len = sp_io::storage::read(&key, &mut [0; 0], 0).ok_or(Error::<T>::ProposalMissing)?;
			ensure!(proposal_len <= length_bound, Error::<T>::WrongProposalLength);
			let proposal = ProposalOf::<T>::get(currency_id, hash).ok_or(Error::<T>::ProposalMissing)?;
			let proposal_weight = proposal.get_dispatch_info().weight;
			ensure!(proposal_weight <= weight_bound, Error::<T>::WrongProposalWeight);
			Ok((proposal, proposal_len as usize))
		}

		fn do_approve_proposal(
			currency_id: CurrencyIdOf<T>,
			seats: MultiBalanceOf<T>,
			yes_votes: MultiBalanceOf<T>,
			proposal_hash: T::Hash,
			proposal: <T as Config>::Proposal,
		) -> (Weight, u32) {
			Self::deposit_event(Event::Approved(proposal_hash));

			let dispatch_weight = proposal.get_dispatch_info().weight;

			let origin = IcoRawOrigin::Members(yes_votes, seats).into();

			let result = proposal.dispatch(origin);
			Self::deposit_event(Event::Executed(proposal_hash, result.map(|_| ()).map_err(|e| e.error)));
			// default to the dispatch info weight for safety
			let proposal_weight = get_result_weight(result).unwrap_or(dispatch_weight); // P1

			let proposal_count = Self::remove_proposal(currency_id, proposal_hash);
			(proposal_weight, proposal_count)
		}

		fn do_disapprove_proposal(currency_id: CurrencyIdOf<T>, proposal_hash: T::Hash) -> u32 {
			// disapproved
			Self::deposit_event(Event::Disapproved(proposal_hash));
			Self::remove_proposal(currency_id, proposal_hash)
		}

		// Removes a proposal from the pallet, cleaning up votes and the vector of
		// proposals.
		fn remove_proposal(currency_id: CurrencyIdOf<T>, proposal_hash: T::Hash) -> u32 {
			// remove proposal and vote
			ProposalOf::<T>::remove(currency_id, &proposal_hash);
			Voting::<T>::remove(currency_id, &proposal_hash);
			let num_proposals = Proposals::<T>::mutate(currency_id, |proposals| {
				proposals.retain(|h| h != &proposal_hash);
				proposals.len() + 1 // calculate weight based on original length
			});
			num_proposals as u32
		}
	}

	pub(crate) fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
		match result {
			Ok(post_info) => post_info.actual_weight,
			Err(err) => err.post_info.actual_weight,
		}
	}

	/// Origin for the dao module.
	#[pallet::origin]
	pub type Origin<T> = IcoRawOrigin<<T as frame_system::Config>::AccountId, MultiBalanceOf<T>>;

	pub struct EnsureProportionAtLeast<T: Config, N: U32, D: U32, AccountId>(
		sp_std::marker::PhantomData<(T, N, D, AccountId)>,
	);

	impl<
			T: Config,
			O: Into<Result<IcoRawOrigin<AccountId, MultiBalanceOf<T>>, O>>
				+ From<IcoRawOrigin<AccountId, MultiBalanceOf<T>>>,
			N: U32,
			D: U32,
			AccountId,
		> EnsureOrigin<O> for EnsureProportionAtLeast<T, N, D, AccountId>
	{
		type Success = ();
		fn try_origin(o: O) -> Result<Self::Success, O> {
			o.into().and_then(|o| match o {
				IcoRawOrigin::Members(n, m)
					if n * D::VALUE.saturated_into::<MultiBalanceOf<T>>()
						>= N::VALUE.saturated_into::<MultiBalanceOf<T>>() * m =>
				{
					Ok(())
				}
				r => Err(O::from(r)),
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin() -> O {
			O::from(IcoRawOrigin::Members(MultiBalanceOf::<T>::from(1u32), MultiBalanceOf::<T>::from(1u32)))
		}
	}
}
