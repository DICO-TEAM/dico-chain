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
use codec::{self, Codec, Decode, Encode};
use jsonrpc_core::{
	futures::future::{self as rpc_future},
	Error as RpcError, ErrorCode, Result
};
use jsonrpc_derive::rpc;
use pallet_ico_rpc_runtime_api::IcoAmountApi;
use sc_rpc_api::DenyUnsafe;
use sp_blockchain::{Error as ClientError, HeaderBackend};
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits};
use std::convert::TryInto;
use std::sync::Arc;
// type Result<T> = Box<dyn rpc_future::Future<Item = T, Error = RpcError> + Send>;

/// Ico RPC method
#[rpc]
pub trait IcoApi<AccountId, CurrencyId, Index, Balance> {
	#[rpc(name = "ico_canReleaseAmount", alias("canReleaseAmount"))]
	fn can_release_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<NumberOrHex>;
	#[rpc(name = "ico_getRewardAmount", alias("getRewardAmount"))]
	fn get_reward_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index)
		-> Result<NumberOrHex>;
	#[rpc(name = "ico_canUnlockAmount", alias("canUnlockAmount"))]
	fn can_unlock_amount(&self, user: AccountId, currency_id: CurrencyId, index: Index) -> Result<NumberOrHex>;
	#[rpc(name = "ico_canJoinAmount", alias("canJoinAmount"))]
	fn can_join_amount(
		&self,
		user: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<(NumberOrHex, NumberOrHex)>;
	#[rpc(name = "ico_getTokenPrice", alias("getTokenPrice"))]
	fn get_token_price(&self, currency_id: CurrencyId) -> Result<NumberOrHex>;
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

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, AccountId, CurrencyId, Index, Balance, Block> IcoApi<AccountId, CurrencyId, Index, Balance>
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
	Balance: Codec + traits::MaybeDisplay + Copy + TryInto<NumberOrHex> + std::marker::Send + 'static,
{
	fn can_release_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<NumberOrHex> {
		let get_release_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_release_amount(&at, account, currency_id, index)
				.map_err(|e| RpcError {
					code: ErrorCode::ServerError(Error::RuntimeError.into()),
					message: "Unable to dry run extrinsic.".into(),
					data: Some(format!("{:?}", e).into()),
				})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| RpcError {
					code: ErrorCode::InvalidParams,
					message: format!("{} doesn't fit in NumberOrHex representation", value),
					data: None,
				})
			};
			try_into_rpc_balance(amount)
		};

		get_release_amount()
	}

	fn get_token_price(&self, currency_id: CurrencyId) -> Result<NumberOrHex> {
		let get_token_price = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api.get_token_price(&at, currency_id).map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to dry run extrinsic.".into(),
				data: Some(format!("{:?}", e).into()),
			})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| RpcError {
					code: ErrorCode::InvalidParams,
					message: format!("{} doesn't fit in NumberOrHex representation", value),
					data: None,
				})
			};
			try_into_rpc_balance(amount)
		};

		get_token_price()
	}

	fn can_join_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<(NumberOrHex, NumberOrHex)> {
		let can_join_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_join_amount(&at, account, currency_id, index)
				.map_err(|e| RpcError {
					code: ErrorCode::ServerError(Error::RuntimeError.into()),
					message: "Unable to dry run extrinsic.".into(),
					data: Some(format!("{:?}", e).into()),
				})?;

			let try_into_rpc_balance = |value: Vec<Balance>| {
				let mut new_value = vec![];
				for i in value {
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
					Err(RpcError {
						code: ErrorCode::InvalidParams,
						message: format!("doesn't fit in NumberOrHex representation"),
						data: None,
					})
				}
			};
			try_into_rpc_balance(vec![amount.0, amount.1])
		};

		can_join_amount()
	}

	fn get_reward_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<NumberOrHex> {
		let get_reward_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.get_reward_amount(&at, account, currency_id, index)
				.map_err(|e| RpcError {
					code: ErrorCode::ServerError(Error::RuntimeError.into()),
					message: "Unable to dry run extrinsic.".into(),
					data: Some(format!("{:?}", e).into()),
				})?;

			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| RpcError {
					code: ErrorCode::InvalidParams,
					message: format!("{} doesn't fit in NumberOrHex representation", value),
					data: None,
				})
			};
			try_into_rpc_balance(amount)
		};

		get_reward_amount()
	}

	fn can_unlock_amount(
		&self,
		account: AccountId,
		currency_id: CurrencyId,
		index: Index,
	) -> Result<NumberOrHex> {
		let get_unlock_amount = || {
			let api = self.client.runtime_api();
			let best = self.client.info().best_hash;
			let at = BlockId::hash(best);

			let amount = api
				.can_unlock_amount(&at, account, currency_id, index)
				.map_err(|e| RpcError {
					code: ErrorCode::ServerError(Error::RuntimeError.into()),
					message: "Unable to dry run extrinsic.".into(),
					data: Some(format!("{:?}", e).into()),
				})?;
			let try_into_rpc_balance = |value: Balance| {
				value.try_into().map_err(|_| RpcError {
					code: ErrorCode::InvalidParams,
					message: format!("{} doesn't fit in NumberOrHex representation", value),
					data: None,
				})
			};
			try_into_rpc_balance(amount)
		};

		get_unlock_amount()
	}
}
