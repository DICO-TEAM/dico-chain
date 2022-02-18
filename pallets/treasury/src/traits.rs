#![cfg_attr(not(feature = "std"), no_std)]

pub trait DicoTreasuryHandler<AccountId> {
	fn get_treasury_account_id() -> AccountId;
}
