#![allow(clippy::upper_case_acronyms)]

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result as RpcResult};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sc_rpc_api::DenyUnsafe;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay},
};
use std::sync::Arc;
use std::convert::TryInto;
use sp_rpc::number::NumberOrHex;

pub use pallet_farm_rpc_runtime_api::FarmApi as FarmRuntimeApi;


#[rpc]
pub trait FarmApi<AccountId, PoolId, Balance> {
	#[rpc(name="farm_getParticipantReward")]
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

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}


impl<C, Block, AccountId, PoolId, Balance>
FarmApi<AccountId, PoolId, Balance> for Farm<C, Block>
	where
		Block: BlockT,
		C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
		C::Api: FarmRuntimeApi<Block, AccountId, PoolId, Balance>,
		AccountId: Codec,
		PoolId: Codec,
		Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> +  std::marker::Send + 'static,
{
	fn get_participant_reward(
		&self,
		account: AccountId,
		pid: PoolId,
	) -> RpcResult<NumberOrHex> {
		let api = self.client.runtime_api();
		// let at = BlockId::hash(at.unwrap_or_else(||
		// 	// If the block hash is not supplied assume the best block.
		// 	self.client.info().best_hash)
		// );
		let best = self.client.info().best_hash;
		let at = BlockId::hash(best);


		let reward = api.get_participant_reward(&at, account, pid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to retrieve participant reward.".into(),
			data: Some(format!("{:?}", e).into()),
		})?;

		reward.try_into().map_err(|_| RpcError {
			code: ErrorCode::ServerError(Error::DecodeError.into()),
			message: "Unable to decode participant reward.".into(),
			data: None,
		})
	}
}





