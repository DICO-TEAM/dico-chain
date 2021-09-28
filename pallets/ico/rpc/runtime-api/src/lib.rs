// Copyright 2021 DICO  Developer.
// This file is part of DICO

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
		fn can_join_amount(user: AccountId, currency_id: CurrencyId, index: Index) -> (Balance, Balance);
		fn get_token_price(currency_id: CurrencyId) -> Balance;
	}

}
