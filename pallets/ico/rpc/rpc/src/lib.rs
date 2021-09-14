// #![cfg_attr(not(feature = "std"), no_std)]
use jsonrpc_derive::rpc;
use jsonrpc_core::{Error as RpcError, {futures::future::{self as rpc_future, result}}, ErrorCode};
use sp_runtime::{
    generic::BlockId,
    traits,
};
use sp_blockchain::{
    HeaderBackend,
    Error as ClientError
};
use std::convert::TryInto;
use codec::{self, Codec, Decode, Encode};
use std::sync::Arc;
use sc_rpc_api::DenyUnsafe;
use pallet_ico_rpc_runtime_api::IcoAmountApi;
use sp_rpc::number::NumberOrHex;
type FutureResult<T> = Box<dyn rpc_future::Future<Item = T, Error = RpcError> + Send>;

/// Ico RPC method
#[rpc]
pub trait IcoApi<AccountId, CurrencyId, Index, Balance> {
    #[rpc(name = "ico_canReleaseAmount", alias("canReleaseAmount"))]
    fn can_release_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex>;
    #[rpc(name = "ico_getRewardAmount", alias("getRewardAmount"))]
    fn get_reward_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex>;
    #[rpc(name = "ico_canUnlockAmount", alias("canUnlockAmount"))]
    fn can_unlock_amount(&self, user: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex>;
    #[rpc(name = "ico_canJoinAmount", alias("canJoinAmount"))]
    fn can_join_amount(&self, user: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex>;
}

pub struct FullIco<C, B> {
    client: Arc<C>,
    deny_unsafe: DenyUnsafe,
    _marker: std::marker::PhantomData<B>,
}

impl <C, B> FullIco<C, B> {
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


impl<C, AccountId, CurrencyId, Index, Balance, Block> IcoApi<AccountId, CurrencyId, Index, Balance> for FullIco<C, Block>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: IcoAmountApi<Block, AccountId, CurrencyId, Index, Balance>,
    Block: traits::Block,
    AccountId: Clone + std::fmt::Display + Codec,
    Index: Clone + std::fmt::Display + Codec + Send + traits::AtLeast32Bit + 'static,
    CurrencyId: Clone + std::fmt::Display + Codec,
    Balance: Codec + traits::MaybeDisplay + Copy + TryInto<NumberOrHex> +  std::marker::Send + 'static,
{
    fn can_release_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex> {
        let get_release_amount = || {
            let api = self.client.runtime_api();
            let best = self.client.info().best_hash;
            let at = BlockId::hash(best);

            let amount = api.can_release_amount(&at, account, currency_id, index).map_err(|e| RpcError {
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

        Box::new(result(get_release_amount()))
    }

    fn can_join_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex> {
        let can_join_amount = || {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let amount = api.can_join_amount(&at, account, currency_id, index).map_err(|e| RpcError {
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

        Box::new(result(can_join_amount()))
    }

    fn get_reward_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex> {
        let get_reward_amount = || {
            let api = self.client.runtime_api();
            let best = self.client.info().best_hash;
            let at = BlockId::hash(best);

            let amount = api.get_reward_amount(&at, account, currency_id, index).map_err(|e| RpcError {
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

        Box::new(result(get_reward_amount()))
    }

    fn can_unlock_amount(&self, account: AccountId, currency_id: CurrencyId, index: Index) -> FutureResult<NumberOrHex
    > {
        let get_unlock_amount = || {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;
        let at = BlockId::hash(best);

        let amount = api.can_unlock_amount(&at, account, currency_id, index).map_err(|e| RpcError {
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

    Box::new(result(get_unlock_amount()))
    }
}






