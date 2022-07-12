#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::result;

pub trait CurrenciesHandler<CurrencyId, DicoAssetMetadata, DispatchErr, AccountId, Balance, DispatchResult> {
	fn get_metadata(currency: CurrencyId) -> result::Result<DicoAssetMetadata, DispatchErr>;
	fn do_create(
		user: AccountId,
		currency_id: CurrencyId,
		metadata: Option<DicoAssetMetadata>,
		amount: Balance,
		is_swap_deposit: bool,
	) -> DispatchResult;
}

pub trait AssetIdMapping<CurrencyId, MultiLocation> {
	fn get_multi_location(asset_id: CurrencyId) -> Option<MultiLocation>;
	fn get_currency_id(multi_location: MultiLocation) -> Option<CurrencyId>;
	fn get_weight_rate_multiple(location: MultiLocation) -> Option<u128>;
}
