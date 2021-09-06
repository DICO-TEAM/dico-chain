#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};

pub trait CurrenciesHandler<CurrencyId, DicoAssetMetadata, DispatchErr, AccountId, Balance, DispatchResult> {
	fn get_metadata(currency: CurrencyId) -> result::Result<DicoAssetMetadata, DispatchErr>;
	fn do_deposit(user: AccountId, currency_id: CurrencyId, amount: Balance, is_swap_deposit: bool) -> DispatchResult;
}
