#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};

pub trait CurrenciesHandler<CurrencyId, DicoAssetMetadata, DispatchErr> {
	fn get_metadata(currency: CurrencyId) -> result::Result<DicoAssetMetadata, DispatchErr>;
}
