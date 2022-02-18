#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};
pub trait IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> {
	fn set_ico_for_bench(
		currency_id: CurrencyId,
		index: u32,
		initiator: AccountId,
		joiner: AccountId,
		joiner1: AccountId,
	) -> DispatchResult;
	fn is_project_ico_member(currency_id: CurrencyId, index: u32, who: &AccountId) -> result::Result<bool, DispathErr>;
	fn get_user_total_amount(currency_id: CurrencyId, index: u32, who: &AccountId) -> MulBalanceOf;
	fn get_project_total_ico_amount(currency_id: CurrencyId, index: u32) -> result::Result<MulBalanceOf, DispathErr>;
}
use frame_support::Parameter;
use sp_runtime::DispatchResult;
// pub use dico_primitives::{Balance, AccountId};

impl<
		CurrencyId: Ord + Clone,
		MulBalanceOf: Ord + Clone + Default + From<u32>,
		AccountId: Ord + Clone,
		DispathErr: Clone,
		BlockNumber: Ord + Clone + Default,
	> IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> for ()
{
	fn set_ico_for_bench(
		currency_id: CurrencyId,
		index: u32,
		account_id: AccountId,
		joiner: AccountId,
		joiner1: AccountId,
	) -> DispatchResult {
		Ok(())
	}

	fn is_project_ico_member(_: CurrencyId, _: u32, _: &AccountId) -> Result<bool, DispathErr> {
		Ok(true)
	}

	fn get_user_total_amount(_: CurrencyId, _: u32, _: &AccountId) -> MulBalanceOf {
		MulBalanceOf::from(100u32)
	}

	fn get_project_total_ico_amount(currency_id: CurrencyId, index: u32) -> Result<MulBalanceOf, DispathErr> {
		Ok(MulBalanceOf::from(500u32))
	}
}

pub trait PowerHandler<AccountId, DispathResult, Balance> {
	fn sub_user_power(user: &AccountId, amount: Balance) -> DispathResult;
	fn add_user_power(user: &AccountId, amount: Balance) -> DispathResult;
}

impl<AccountId: Default + Parameter, Balance: Default + Parameter> PowerHandler<AccountId, DispatchResult, Balance>
	for ()
{
	fn sub_user_power(user: &AccountId, amount: Balance) -> DispatchResult {
		Ok(())
	}

	fn add_user_power(user: &AccountId, amount: Balance) -> DispatchResult {
		Ok(())
	}
}
