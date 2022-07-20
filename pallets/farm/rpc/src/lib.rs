#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use codec::Codec;
// use jsonrpc_core::{Error as RpcError, ErrorCode, Result as RpcResult};
// use jsonrpc_derive::rpc;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay},
};
use std::convert::TryInto;
use std::sync::Arc;

pub use pallet_farm_rpc_runtime_api::FarmApi as FarmRuntimeApi;

#[rpc(client, server)]
pub trait FarmApi<AccountId, PoolId, Balance> {
	#[method(name = "farm_getParticipantReward")]
	fn get_participant_reward(&self, account: AccountId, pid: PoolId) -> RpcResult<NumberOrHex>;
}

/// A struct that implements the [`FarmApi`].
pub struct Farm<C, B> {
	client: Arc<C>,
	deny_unsafe: DenyUnsafe,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Farm<C, B> {
	/// Create new `Farm` with the given reference to the client.
	pub fn new(client: Arc<C>, deny_unsafe: DenyUnsafe) -> Self {
		Farm {
			client,
			deny_unsafe,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
	/// The transaction was not decodable.
	DecodeError,
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
impl<C, Block, AccountId, PoolId, Balance> FarmApiServer<AccountId, PoolId, Balance> for Farm<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: FarmRuntimeApi<Block, AccountId, PoolId, Balance>,
	AccountId: Codec,
	PoolId: Codec,
	Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + std::marker::Send + 'static,
{
	fn get_participant_reward(&self, account: AccountId, pid: PoolId) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);

		let reward = api.get_participant_reward(&at, account, pid).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to query participant reward.",
				Some(format!("{:?}", e)),
			))
		})?;

		reward.try_into().map_err(|_| {
			JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
				ErrorCode::InvalidParams.code(),
				format!("doesn't fit in NumberOrHex representation"),
				None::<()>,
			)))
		})
	}
}
