#![cfg_attr(not(feature = "std"), no_std)]
sp_api::decl_runtime_apis! {
	pub trait IcoAmountApi<AccountId, CurrencyId, Index, Balance> where
		AccountId: codec::Codec,
		CurrencyId: codec::Codec,
		Index: codec::Codec,
		Balance: codec::Codec,
	{
		fn can_release_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance;
		fn get_reward_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance;
		fn can_unlock_amount(user: AccountId, currency_id: CurrencyId, index: Index) -> Balance;
		fn can_join_amount(user: AccountId, currency_id: CurrencyId, index: Index) -> Balance;
	}

}
