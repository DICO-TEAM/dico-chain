#![allow(unused_assignments)]
#![allow(unreachable_patterns)]

use super::*;
pub use codec::MaxEncodedLen;
use daos_create_dao;
use daos_square::{
	traits::{ConvertInto, Pledge as PledgeTrait},
	Error,
};
use daos_primitives::{
	ids::{DaoId, Fungible, Nft as NFT},
	traits::{AfterCreate, BaseCallFilter, TryCreate},
	types::MemberCount,
	AccountIdConversion, TrailingZeroInput,
};
pub use frame_support::{
	codec::{Decode, Encode},
	parameter_types,
};
use orml_traits::MultiReservableCurrency;
pub use scale_info::TypeInfo;
use sp_runtime::DispatchError;
pub use sp_runtime::{traits::Hash, RuntimeDebug};

type CallId = u32;
impl TryFrom<Call> for CallId {
	type Error = ();

	fn try_from(call: Call) -> Result<Self, Self::Error> {
		match call {
			// daos
			Call::CreateDao(func) => match func {
				daos_create_dao::Call::dao_remark { .. } => Ok(101 as CallId),
				_ => Err(()),
			},
			Call::DaoAgency(func) => match func {
				daos_agency::Call::disapprove_proposal { .. } => Ok(201 as CallId),
				daos_agency::Call::set_motion_duration { .. } => Ok(202 as CallId),
				daos_agency::Call::set_max_proposals { .. } => Ok(203 as CallId),
				daos_agency::Call::set_max_members { .. } => Ok(204 as CallId),
				daos_agency::Call::set_ensure_origin_for_every_call { .. } => Ok(205 as CallId),
				_ => Err(()),
			},
			Call::DaoSquare(func) => match func {
				daos_square::Call::set_min_vote_weight_for_every_call { .. } => Ok(301 as CallId),
				daos_square::Call::set_max_public_props { .. } => Ok(302 as CallId),
				daos_square::Call::set_launch_period { .. } => Ok(303 as CallId),
				daos_square::Call::set_minimum_deposit { .. } => Ok(304 as CallId),
				daos_square::Call::set_voting_period { .. } => Ok(305 as CallId),
				daos_square::Call::set_rerserve_period { .. } => Ok(306 as CallId),
				daos_square::Call::set_enactment_period { .. } => Ok(307 as CallId),
				_ => Err(()),
			},
			Call::DaoSudo(func) => match func {
				daos_sudo::Call::set_sudo_account { .. } => Ok(401 as CallId),
				daos_sudo::Call::close_sudo { .. } => Ok(402 as CallId),
				_ => Err(()),
			},

			// about assets
			Call::Currencies(func) => match func {
				pallet_currencies::Call::burn { .. } => Ok(901 as CallId),
				pallet_currencies::Call::transfer { .. } => Ok(902 as CallId),
				_ => Err(()),
			},
			Call::Nft(func) => match func {
				pallet_nft::Call::transfer { .. } => Ok(501 as CallId),
				pallet_nft::Call::claim { .. } => Ok(502 as CallId),
				pallet_nft::Call::burn { .. } => Ok(503 as CallId),
				pallet_nft::Call::offer_token_for_sale { .. } => Ok(504 as CallId),
				pallet_nft::Call::withdraw_sale { .. } => Ok(505 as CallId),
				pallet_nft::Call::buy_token { .. } => Ok(506 as CallId),
				pallet_nft::Call::active { .. } => Ok(507 as CallId),
				pallet_nft::Call::inactive { .. } => Ok(508 as CallId),
				_ => Err(()),
			},
			Call::AMM(func) => match func {
				pallet_amm::Call::add_liquidity { .. } => Ok(601 as CallId),
				pallet_amm::Call::remove_liquidity { .. } => Ok(602 as CallId),
				pallet_amm::Call::swap_exact_assets_for_assets { .. } => Ok(603 as CallId),
				pallet_amm::Call::swap_assets_for_exact_assets { .. } => Ok(604 as CallId),
				_ => Err(()),
			},

			// vc
			Call::Vault(func) => match func {
				pallet_vc::Call::set_guarders { .. } => Ok(701 as CallId),
				pallet_vc::Call::remove_guarder { .. } => Ok(702 as CallId),
				pallet_vc::Call::add_guarder { .. } => Ok(703 as CallId),
				pallet_vc::Call::unreserve { .. } => Ok(704 as CallId),
				pallet_vc::Call::set_fee { .. } => Ok(705 as CallId),
				pallet_vc::Call::open_cex_transfer { .. } => Ok(706 as CallId),
				pallet_vc::Call::close_cex_transfer { .. } => Ok(707 as CallId),
				_ => Err(()),
			},
			// ico
			Call::Ico(func) => match func {
				pallet_ico::Call::join { .. } => Ok(801 as CallId),
				pallet_ico::Call::user_release_ico_amount { .. } => Ok(802 as CallId),
				pallet_ico::Call::unlock { .. } => Ok(803 as CallId),
				pallet_ico::Call::get_reward { .. } => Ok(804 as CallId),
				_ => Err(()),
			},

			_ => Err(()),
		}
	}
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum ConcreteId<ClassId, TokenId> {
	NftClassId(ClassId),
	FungibleTokenId(TokenId),
}

impl<T: Encode + Decode, ClassId: Encode + Decode, TokenId: Encode + Decode> AccountIdConversion<T>
	for ConcreteId<ClassId, TokenId>
{
	fn into_account(&self) -> T {
		match self {
			ConcreteId::NftClassId(x) => (b"nft ", NFT(x))
				.using_encoded(|b| T::decode(&mut TrailingZeroInput(b)))
				.unwrap(),
			ConcreteId::FungibleTokenId(x) => (b"fung", Fungible(x))
				.using_encoded(|b| T::decode(&mut TrailingZeroInput(b)))
				.unwrap(),
		}
	}

	fn try_from_account(x: &T) -> Option<Self> {
		x.using_encoded(|d| {
			if &d[0..4] != b"nft " && &d[0..4] != b"fung" {
				return None;
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

impl<NftId: Default, TokenId: Default> Default for ConcreteId<NftId, TokenId> {
	fn default() -> Self {
		ConcreteId::NftClassId(NftId::default())
	}
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum Pledge<TokenId, Balance> {
	NftTokenId(TokenId),
	FungibleAmount(Balance),
}

impl Default for Pledge<u32, Balance> {
	fn default() -> Self {
		Pledge::FungibleAmount(0 as u128)
	}
}
impl PledgeTrait<Balance, AccountId, DaoId, Conviction, BlockNumber, DispatchError> for Pledge<u32, Balance> {
	fn try_vote(
		&self,
		who: &AccountId,
		dao_id: &DaoId,
		conviction: &Conviction,
	) -> Result<(Balance, BlockNumber), DispatchError> {
		if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
			return Ok((Default::default(), Default::default()));
		}
		let mut amount = 0 as Balance;
		let concrete_id = daos_create_dao::Pallet::<Runtime>::try_get_concrete_id(*dao_id)?;
		match self {
			Pledge::FungibleAmount(x) => {
				if let ConcreteId::FungibleTokenId(id) = concrete_id {
					Currencies::reserve(id, &who, *x)?;
					amount = *x;
					return Ok((
						amount
							.checked_mul(conviction.convert_into())
							.ok_or(daos_square::Error::<Runtime>::Overflow)?,
						conviction.convert_into(),
					));
				}
			}
			Pledge::NftTokenId(x) => {
				if let ConcreteId::NftClassId(class_id) = concrete_id {
					Nft::try_lock(&who, (class_id, *x))?;
					amount = DOLLARS;
					return Ok((
						DOLLARS
							.checked_mul(conviction.convert_into())
							.ok_or(daos_square::Error::<Runtime>::Overflow)?,
						conviction.convert_into(),
					));
				}
			}
		}
		Err(daos_square::Error::<Runtime>::PledgeNotEnough)?
	}

	fn vote_end_do(&self, who: &AccountId, dao_id: &DaoId) -> Result<(), DispatchError> {
		if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
			return Ok(());
		}
		let concrete_id = daos_create_dao::Pallet::<Runtime>::try_get_concrete_id(*dao_id)?;
		match self {
			Pledge::FungibleAmount(x) => {
				if let ConcreteId::FungibleTokenId(id) = concrete_id {
					Currencies::reserve(id, &who, *x)?;
					return Ok(());
				}
			}
			Pledge::NftTokenId(x) => {
				if let ConcreteId::NftClassId(class_id) = concrete_id {
					Nft::try_unlock(&who, (class_id, *x))?;
					return Ok(());
				}
			}
		}
		Err(daos_square::Error::<Runtime>::PledgeNotEnough)?
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
			_ => 120 * DAYS,
		}
	}
}

impl TryCreate<AccountId, DaoId, DispatchError> for ConcreteId<u32, CurrencyId> {
	fn try_create(&self, who: AccountId, dao_id: DaoId) -> Result<(), DispatchError> {
		match &self {
			ConcreteId::FungibleTokenId(token_id) => {
				Currencies::try_create_dao(&who, *token_id, dao_id)?;
			}
			ConcreteId::NftClassId(class_id) => {
				Nft::try_create_dao(&who, *class_id, dao_id)?;
			}
		}
		Ok(())
	}
}

impl BaseCallFilter<Call> for ConcreteId<u32, CurrencyId> {
	fn contains(&self, call: Call) -> bool {
		true
	}
}

// Set root account for sudo module.
pub struct CreatedDo;
impl AfterCreate<AccountId, DaoId> for CreatedDo {
	fn do_something(a: AccountId, b: DaoId) {
		daos_sudo::Account::<Runtime>::insert(b, a);
	}
}

impl daos_create_dao::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type CallId = CallId;
	type DaoId = DaoId;
	type ConcreteId = ConcreteId<u32, CurrencyId>;
	type AfterCreate = CreatedDo;
	type WeightInfo = ();
}

impl daos_sudo::Config for Runtime {
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxMembersForSystem: MemberCount = 20;
}

pub struct CollectiveBaseCallFilter;

impl Contains<Call> for CollectiveBaseCallFilter {
	fn contains(call: &Call) -> bool {
		if let Call::DoAs(func) = call {
			matches!(func, daos_doas::Call::do_as_agency { .. })
		} else {
			false
		}
	}
}

impl daos_agency::Config for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type CollectiveBaseCallFilter = CollectiveBaseCallFilter;
	type Event = Event;
	type DefaultVote = daos_agency::PrimeDefaultVote;
	type MaxMembersForSystem = MaxMembersForSystem;
	type WeightInfo = ();
	// type WeightInfo = ();
}

impl daos_doas::Config for Runtime {
	type Event = Event;
	type DoAsOrigin = DaoAgency;
	type WeightInfo = ();
}

impl pallet_vc::Config for Runtime {
	type Event = Event;
	type SetCollectiveMembers = DaoAgency;
	type MultiCurrency = Currencies;
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl daos_square::Config for Runtime {
	type Event = Event;
	type Pledge = Pledge<u32, Balance>;
	type Conviction = Conviction;
	type Currency = Balances;
	type WeightInfo = ();
}
