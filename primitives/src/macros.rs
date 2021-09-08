//! A set of macro used in pallets.
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_export]
macro_rules! to_u256 {
    ($($x:expr),+) => (
        {($(U256::from($x)),+)}
    );
}

#[macro_export]
macro_rules! to_balance {
	($x:expr) => {
		Balance::try_from($x).map_err(|_| ArithmeticError::Overflow)
	};
}
