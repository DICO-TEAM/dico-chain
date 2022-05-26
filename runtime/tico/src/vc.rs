
use super::*;
use daos_create_dao;
use daos_primitives::{dao_id::DaoId, traits::{Checked, BaseDaoCallFilter}, types::MemberCount};
pub use codec::MaxEncodedLen;
pub use frame_support::{codec::{Decode, Encode}, parameter_types};
pub use sp_runtime::{traits::Hash, RuntimeDebug};
pub use scale_info::TypeInfo;
use sp_runtime::DispatchError;

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
						daos_collective::Call::set_ensure_for_every_call{..} => Ok(1 as CallId),
					_ => Err(()),
				}
			},
			Call::Vault(func) => Ok(2 as CallId),
			Call::Nft(_) => Ok(3 as CallId),
			Call::AMM(_) => Ok(4 as CallId),
			Call::Currencies(_) => Ok(5 as CallId),
			_ => Err(()),
		}
	}
}


#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum SecondId<NftId, TokenId> {
	Nft(NftId),
	Currency(TokenId)
}

impl<NftId: Default, TokenId: Default> Default for SecondId<NftId, TokenId> {
	fn default() -> Self {
		SecondId::Nft(NftId::default())
	}
}


impl Checked<AccountId, DispatchError> for SecondId<u32, CurrencyId>{
	fn is_can_create(&self, who: AccountId) -> Result<(), DispatchError> {
		match &self {
			SecondId::Currency(token_id) => {
				if !Currencies::is_owner(*token_id, &who) {
					return Err(pallet_currencies::Error::<Runtime>::NotOwner)?;
				}

			},
			SecondId::Nft(class_id) => {
				if !Nft::is_issuer(&who, *class_id) {
					return Err(pallet_nft::Error::<Runtime>::NotIssuer)?;
				}
			},
		}
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
		if let Call::DoAS(func) = call {
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
	type WeightInfo = ();
}


impl daos_doas::Config for Runtime {
	type Event = Event;
	type DoAsOrigin = DaoCollective;
}

impl pallet_vc::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Currencies;
}



