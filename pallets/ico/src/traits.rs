#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};
pub trait IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> {
	fn is_project_ico_member(currency_id: CurrencyId, index: u32, who: &AccountId) -> result::Result<bool, DispathErr>;
	fn get_user_total_amount(currency_id: CurrencyId, index: u32, who: &AccountId) -> MulBalanceOf;
	fn get_project_total_ico_amount(currency_id: CurrencyId, index: u32) -> result::Result<MulBalanceOf, DispathErr>;
}

pub trait PowerHandler<AccountId, DispathResult, Balance> {
	fn sub_user_power(user: &AccountId, amount: Balance) -> DispathResult;
	fn add_user_power(user: &AccountId, amount: Balance) -> DispathResult;
}
