use sp_runtime::{RuntimeDebug, ArithmeticError};
use sp_core::U256;

pub trait PriceProvider<CurrencyId> {
    type Price;
    fn get_price_from_swap(currency_id1: CurrencyId, currency_id2: CurrencyId) -> sp_std::result::Result<U256, ArithmeticError>;
}