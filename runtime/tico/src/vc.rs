#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

use super::*;
use daos_create_dao;
use orml_traits::MultiReservableCurrency;
use daos_primitives::{ids::{DaoId, Nft as NFT, Fungible}, traits::{Checked, BaseDaoCallFilter}, types::MemberCount, AccountIdConversion, TrailingZeroInput};
pub use codec::MaxEncodedLen;
pub use frame_support::{codec::{Decode, Encode}, parameter_types};
pub use sp_runtime::{traits::Hash, RuntimeDebug};
pub use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use daos_democracy::{Error, traits::{Vote as VoteTrait, CheckedVote, ConvertInto}};

type CallId = u32;
impl TryFrom<Call> for CallId {
	type Error = ();

	fn try_from(call: Call) -> Result<Self, Self::Error> {
		match call {
			Call::DaoCollective(func) => {
				match func {
					daos_collective::Call::disapprove_proposal{..} |
						daos_collective::Call::set_motion_duration{..} |
						daos_collective::Call::set_max_proposals{..} |
						daos_collective::Call::set_max_members{..} |
						daos_collective::Call::set_ensure_for_every_call{..} => Ok(100 as CallId),
					_ => Err(()),
				}
			},
			Call::Vault(func) => Ok(200 as CallId),
			Call::Nft(_) => Ok(300 as CallId),
			Call::AMM(_) => Ok(400 as CallId),
			Call::Currencies(_) => Ok(500 as CallId),
			Call::DaoDemocracy(_) => Ok(600 as CallId),
			Call::CreateDao(_) => Ok(700 as CallId),
			_ => Err(()),
		}
	}
}


#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum SecondId<ClassId, TokenId> {
	NftClassId(ClassId),
	FungibleTokenId(TokenId)
}

impl<T: Encode + Decode, ClassId: Encode + Decode, TokenId: Encode + Decode> AccountIdConversion<T> for SecondId<ClassId, TokenId> {
	fn into_account(&self) -> T {
		match self {
			SecondId::NftClassId(x) => (b"nft ", NFT(x)).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap(),
			SecondId::FungibleTokenId(x) => (b"fung", Fungible(x)).using_encoded(|b| T::decode(&mut TrailingZeroInput(b))).unwrap(),
		}
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"nft " && &d[0..4] != b"fung"{
				return None
			}
			let mut cursor = &d[4..];
			let result = Decode::decode(&mut cursor).ok()?;
			if cursor.iter().all(|x| *x == 0) {
				Some(result)
			} else {
				None
			}
		})
	}
}


impl<NftId: Default, TokenId: Default> Default for SecondId<NftId, TokenId> {
	fn default() -> Self {
		SecondId::NftClassId(NftId::default())
	}
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum Vote<TokenId, Balance> {
	NftTokenId(TokenId),
	FungibleAmount(Balance)
}

impl Default for Vote<u32, Balance> {
	fn default() -> Self {
		Vote::FungibleAmount(0 as u128)
	}
}
impl VoteTrait<Balance, AccountId, SecondId<u32, u32>, Conviction, BlockNumber, DispatchError> for Vote<u32, Balance>{
	fn try_vote(&self, who: &AccountId, second_id: &SecondId<u32, u32>, conviction: &Conviction) -> Result<(Balance, BlockNumber), DispatchError> {
		let mut amount = 0 as Balance;
		match self {
			Vote::FungibleAmount(x) => {
				 if let SecondId::FungibleTokenId(id) = second_id {
					 Currencies::reserve(*id, &who, *x)?;
					 amount = *x;
					return Ok((amount.checked_mul(conviction.convert_into()).ok_or(daos_democracy::Error::<Runtime>::Overflow)?, conviction.convert_into()));
				 }
			},
			Vote::NftTokenId(x) => {
				if let SecondId::NftClassId(class_id) = second_id {
					Nft::try_lock(&who, (*class_id, *x))?;
					amount = DOLLARS;
					return Ok((DOLLARS.checked_mul(conviction.convert_into()).ok_or(daos_democracy::Error::<Runtime>::Overflow)?, conviction.convert_into()))
				}
			},
		}
		Err(daos_democracy::Error::<Runtime>::VoteNotEnough)?
	}

	fn vote_end_do(&self, who: &AccountId, second_id: &SecondId<u32, u32>) -> Result<(), DispatchError> {
		match self {
			Vote::FungibleAmount(x) => {
				if let SecondId::FungibleTokenId(id) = second_id {
					Currencies::reserve(*id, &who, *x)?;
					return Ok(());
				}
			},
			Vote::NftTokenId(x) => {
				if let SecondId::NftClassId(class_id) = second_id {
					Nft::try_unlock(&who, (*class_id, *x))?;
					return Ok(())
				}
			},
		}
		Err(daos_democracy::Error::<Runtime>::VoteNotEnough)?
	}
}

impl CheckedVote<SecondId<u32, u32>, DispatchError> for Vote<u32, Balance> {
	fn is_can_vote(&self, second_id: SecondId<u32, u32>) -> Result<bool, DispatchError> {
		match self {
			Vote::FungibleAmount(x) => {
				if let SecondId::FungibleTokenId(_) = second_id {
					return Ok(true);
				}
			},
			Vote::NftTokenId(x) => {
				if let SecondId::NftClassId(_) = second_id {
					return Ok(true)
				}
			},
		}
		Err(daos_democracy::Error::<Runtime>::VoteError)?
	}
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum Conviction {
	X1,
	X2,
	X3,
	X6,
}

impl Default for Conviction {
	fn default() -> Self {
		Conviction::X1
	}
}

impl ConvertInto<Balance> for Conviction {
	fn convert_into(&self) -> Balance {
		match self {
			Conviction::X1 => 1 as Balance,
			Conviction::X2 => 2 as Balance,
			Conviction::X3 => 3 as Balance,
			Conviction::X6 => 6 as Balance,
			_ => 6 as Balance,
		}
	}
}

impl ConvertInto<BlockNumber> for Conviction {
	fn convert_into(&self) -> BlockNumber {
		match self {
			Conviction::X1 => 30 * DAYS,
			Conviction::X2 => 60 * DAYS,
			Conviction::X3 => 90 * DAYS,
			Conviction::X6 => 120 * DAYS,
			_ => 120 * DAYS
		}
	}
}

impl Checked<AccountId, DaoId, DispatchError> for SecondId<u32, CurrencyId>{
	fn is_can_create(&self, who: AccountId, dao_id: DaoId) -> Result<(), DispatchError> {
		match &self {
			SecondId::FungibleTokenId(token_id) => {
				Currencies::try_create_dao(&who, *token_id, dao_id)?;
			},
			SecondId::NftClassId(class_id) => {
				Nft::try_create_dao(&who, *class_id, dao_id)?;
		} }
		Ok(())
	}
}


impl BaseDaoCallFilter<Call> for SecondId<u32, CurrencyId> {
	fn contains(&self, call: Call) -> bool {
		match call {
			Call::DaoCollective(func) => {
				matches!(func,
					daos_collective::Call::disapprove_proposal{..} |
					daos_collective::Call::set_motion_duration{..} |
					daos_collective::Call::set_max_proposals{..} |
					daos_collective::Call::set_max_members{..} |
					daos_collective::Call::set_ensure_for_every_call{..}
				)
			},
			Call::Vault(_) => true,
			Call::CreateDao(_) => true,
			Call::DaoDemocracy(_) => true,
			Call::Currencies(func) => {
				matches!(func, pallet_currencies::Call::burn{..})
			},
			Call::AMM(func) => true,
			Call::Nft(func) => {
				matches!(func, pallet_nft::Call::buy_token{..} | pallet_nft::Call::burn{..})
			},

			_ => false,
		}
	}
}


impl daos_create_dao::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type CallId = CallId;
	type DaoId = DaoId;
	type SecondId = SecondId<u32, CurrencyId>;
}


impl daos_sudo::Config for Runtime {
	type Event = Event;
}


parameter_types! {
	pub const MaxMembersForSystem: MemberCount = 20;
}

pub struct CollectiveBaseCallFilter;

impl Contains<Call> for CollectiveBaseCallFilter  {
	fn contains(call: &Call) -> bool {
		if let Call::DoAs(func) = call {
			matches!(func, daos_doas::Call::do_as_collective{..})
		}
		else { false }
	}
}

impl daos_collective::Config for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type CollectiveBaseCallFilter = CollectiveBaseCallFilter;
	type Event = Event;
	type DefaultVote = daos_collective::PrimeDefaultVote;
	type GetCollectiveMembers = Vault;
	type MaxMembersForSystem = MaxMembersForSystem;
	// type WeightInfo = ();
}


impl daos_doas::Config for Runtime {
	type Event = Event;
	type DoAsOrigin = DaoCollective;
}

impl pallet_vc::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Currencies;
}

parameter_types! {
	pub const MaxPublicProps: u32 = 20;
	pub const LaunchPeriod: BlockNumber = 2 * HOURS;
	pub const MinimumDeposit: Balance = 10 * DOLLARS;
	pub const VotingPeriod: BlockNumber = 1 * HOURS;
	pub const ReservePeriod: BlockNumber = 1 * DAYS;
	pub const EnactmentPeriod: BlockNumber = HOURS / 2;
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl daos_democracy::Config for Runtime {
	type Event = Event;
	type Vote = Vote<u32, Balance>;
	type Conviction = Conviction;
	type MultiCurrency = Currencies;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type MaxPublicProps = MaxPublicProps;
	type LaunchPeriod = LaunchPeriod;
	type MinimumDeposit = MinimumDeposit;
	type VotingPeriod = VotingPeriod;
	type ReservePeriod = ReservePeriod;
	type EnactmentPeriod = EnactmentPeriod;
}



