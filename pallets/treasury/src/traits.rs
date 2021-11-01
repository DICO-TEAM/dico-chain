#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};

pub trait DicoTreasuryHandler<AccountId> {
	fn get_treasury_account_id() -> AccountId;
}
