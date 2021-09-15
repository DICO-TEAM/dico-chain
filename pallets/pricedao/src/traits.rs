use sp_runtime::{RuntimeDebug, ArithmeticError};
use sp_core::U256;

pub trait PriceProvider<CurrencyId> {
    type Price;
    fn get_price_from_swap(currency_id: CurrencyId,stable_coin: CurrencyId) -> Option<Self::Price>;
    fn get_price_from_oracle(currency_id: CurrencyId) -> Option<Self::Price>;
}

pub trait PriceData<CurrencyId> {
    type Price;
    fn get_price(currency_id: CurrencyId, stable_coin: CurrencyId) -> Option<Self::Price>;
}