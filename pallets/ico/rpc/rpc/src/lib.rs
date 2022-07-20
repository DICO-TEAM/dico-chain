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
// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use codec::{self, Codec, Decode, Encode};
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};

use pallet_ico_rpc_runtime_api::IcoAmountApi;
use sc_rpc_api::DenyUnsafe;
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits};
use std::convert::TryInto;
use std::sync::Arc;
// type Result<T> = Box<dyn rpc_future::Future<Item = T, Error = RpcError> + Send>;

/// Ico RPC method
#[rpc(client, server)]
pub trait IcoApi<AccountId, CurrencyId, Index, Balance> {
	#[method(name = "ico_canReleaseAmount")]
	fn can_release_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex>;
	#[method(name = "ico_getRewardAmount")]
	fn get_reward_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex>;
	#[method(name = "ico_canUnlockAmount")]
	fn can_unlock_amount(&self, user: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex>;
	#[method(name = "ico_canJoinAmount")]
	fn can_join_amount(
		&self,
		user: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> RpcResult<(NumberOrHex, NumberOrHex)>;
	#[method(name = "ico_getTokenPrice")]
	fn get_token_price(&self, currency_id: CurrencyId) -> RpcResult<NumberOrHex>;
}

pub struct FullIco<C, B> {
	client: Arc<C>,
	deny_unsafe: DenyUnsafe,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> FullIco<C, B> {
	pub fn new(client: Arc<C>, deny_unsafe: DenyUnsafe) -> Self {
		FullIco {
			client,
			deny_unsafe,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[async_trait]
impl<C, AccountId, CurrencyId, Index, Balance, Block> IcoApiServer<AccountId, CurrencyId, Index, Balance>
	for FullIco<C, Block>
where
	C: sp_api::ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C: Send + Sync + 'static,
	C::Api: IcoAmountApi<Block, AccountId, CurrencyId, Index, Balance>,
	Block: traits::Block,
	AccountId: Clone + std::fmt::Display + Codec,
	Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
	CurrencyId: Clone + std::fmt::Display + Codec,
	Balance: Codec + traits::MaybeDisplay + Copy + TryInto<NumberOrHex> + std::marker::Send + 'static +  std::fmt::Debug,
{
	fn can_release_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex> {
		// let get_release_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_release_amount(&at, account, currency_id, index)
				.map_err(|e| {
					CallError::Custom(ErrorObject::owned(
					Error::RuntimeError.into(),
					"Unable to query release amount.",
					Some(format!("{:?}", e)),
				))})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| {
					JsonRpseeError::Call(
						CallError::Custom(ErrorObject::owned(
							ErrorCode::InvalidParams.code(),
							format!("{} doesn't fit in NumberOrHex representation", value),
							None::<()>,
						))
					)
				})
			};
			Ok(try_into_rpc_balance(amount)?)
		// };
		//
		// get_release_amount()
	}

	fn get_token_price(&self, currency_id: CurrencyId) -> RpcResult<NumberOrHex> {
		// let get_token_price = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api.get_token_price(&at, currency_id).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query dispatch info.",
				Some(e.to_string()),
			))
		})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| {
				JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					ErrorCode::InvalidParams.code(),
					format!("{} doesn't fit in NumberOrHex representation", value),
					None::<()>,
				)))
			})
			};
			Ok(try_into_rpc_balance(amount)?)
		// };

		// get_token_price()
	}

	fn can_join_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> RpcResult<(NumberOrHex, NumberOrHex)> {
		// let can_join_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_join_amount(&at, account, currency_id, index)
				.map_err(|e| {
					CallError::Custom(ErrorObject::owned(
				Error::DecodeError.into(),
				"Unable to query join amount.",
				Some(format!("{:?}", e)),
			))
				})?;

			let try_into_rpc_balance = |value: Vec<Balance>| {
				let mut new_value = vec![];
				for i in value.clone() {
					let j = i.try_into().ok();
					match j {
						Some(x) => new_value.push(x),
						_ => {}
					}
				}
				if new_value.len() == 2 {
					let mut new_value_iter = new_value.iter();
					let res = (
						new_value_iter.next().unwrap().clone(),
						new_value_iter.next().unwrap().clone(),
					);
					Ok(res)
				} else {
					Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					ErrorCode::InvalidParams.code(),
					format!("{:?} doesn't fit in NumberOrHex representation", value),
					None::<()>,
				))))
				}
			};
			try_into_rpc_balance(vec![amount.0, amount.1])
		// };

		// can_join_amount()
	}

	fn get_reward_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex> {
		// let get_reward_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.get_reward_amount(&at, account, currency_id, index)
				.map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query dispatch info.",
				Some(format!("{:?}", e)),
			))
		})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|e| {
				JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					ErrorCode::InvalidParams.code(),
					format!("{} doesn't fit in NumberOrHex representation", value),
					None::<()>,
				)))
			})
			};
			try_into_rpc_balance(amount)
		// };

		// get_reward_amount()
	}

	fn can_unlock_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> RpcResult<NumberOrHex> {
		// let get_unlock_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_unlock_amount(&at, account, currency_id, index)
				.map_err(|e| {
					CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query unlock amount.",
				Some(e.to_string()),
			))
				})?;
			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| {
					JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
					ErrorCode::InvalidParams.code(),
					format!("{} doesn't fit in NumberOrHex representation", value),
					None::<()>,
				)))
				})
			};
			Ok(try_into_rpc_balance(amount)?)
		// };

		// get_unlock_amount()
	}
}
