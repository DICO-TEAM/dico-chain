//! Runtime API definition for farm pallet.

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait FarmApi<AccountId, PoolId, Balance> where
		AccountId: codec::Codec,
		PoolId: codec::Codec,
		Balance: codec::Codec,
	{
		fn get_participant_reward(account: AccountId, pid: PoolId) -> Balance;
	}
}
