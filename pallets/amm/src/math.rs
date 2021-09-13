#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use sp_core::U256;
use sp_runtime::ArithmeticError;
use frame_support::ensure;

#[cfg(test)]
pub const TEST_DECIMAL: u128 = 10_000_000_000u128;

pub const MINIMUM_LIQUIDITY: u128 = 1000u128;
pub const LIQUIDITY_DECIMALS: u8 = 10;

/// Given a certain amount of asset A and a trading pair reserve,
/// return an equal amount of other asset B.
/// Calculation formula: amount_b = amount_a * reserve_b / reserve_a
/// ******************************************************************************************//
///  quote                                                                                    //
///  aA = amount_a                    aA * rB                                                 //
///  rA = reserve_a          aB = ------------------                                          //
///  rB = reserve_b                     rA                                                    //
///  aB = amount_b                                                                            //
/// ******************************************************************************************//
pub fn quote(
	amount_a: U256,
	reserve_a: U256,
	reserve_b: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	ensure!(!reserve_a.is_zero(), ArithmeticError::DivisionByZero);

	let amount_b = amount_a
		.checked_mul(reserve_b).ok_or(ArithmeticError::Overflow)?
		.checked_div(reserve_a).ok_or(ArithmeticError::Overflow)?;
	Ok(amount_b)
}


/// Given the input amount of the asset and the transaction pair reserve,
/// return the maximum output amount of the other assets.
/// Calculation formula: amountIn * 997/1000 / amountOut = reserveIn / (reserveOut-amountOut)
/// ******************************************************************************************//
///  get_amount_out                                                                           //
///  aI = amount_in                      aI * 997 * rO                                        //
///  rI = reserve_in          aO = --------------------------                                 //
///  rO = reserve_out                 rI * 1000 + aI * 997                                    //
///  aO = amount_out                                                                          //
/// ******************************************************************************************//
pub fn get_amount_out(
	amount_in: U256,
	reserve_in: U256,
	reserve_out: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	let amount_in_with_fee = amount_in
		.checked_mul(U256::from(997)).ok_or(ArithmeticError::Overflow)?;
	let numerator = amount_in_with_fee
		.checked_mul(reserve_out).ok_or(ArithmeticError::Overflow)?;
	let denominator = reserve_in
		.checked_mul(U256::from(1000)).ok_or(ArithmeticError::Overflow)?
		.checked_add(amount_in_with_fee).ok_or(ArithmeticError::Overflow)?;

	ensure!(!denominator.is_zero(), ArithmeticError::DivisionByZero);

	let amount_out = numerator
		.checked_div(denominator).ok_or(ArithmeticError::Overflow)?;

	Ok(amount_out)
}


/// Given the output amount of the asset and the transaction pair reserve,
/// return the required input amount for the other assets.
/// Calculation formula: amountIn * 997/1000 / amountOut = reserveIn / (reserveOut-amountOut)
/// ******************************************************************************************//
///  get_amount_in                                                                            //
///  aI = amount_in                   rI * 1000 * aO                                          //
///  rI = reserve_in          aI = ------------------- + 1                                    //
///  rO = reserve_out                (rO - aO) * 997                                          //
///  aO = amount_out                                                                          //
/// ******************************************************************************************//
pub fn get_amount_in(
	amount_out: U256,
	reserve_in: U256,
	reserve_out: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	let numerator = reserve_in
		.checked_mul(amount_out).ok_or(ArithmeticError::Overflow)?
		.checked_mul(U256::from(1000)).ok_or(ArithmeticError::Overflow)?;
	let denominator = reserve_out
		.checked_sub(amount_out).ok_or(ArithmeticError::Overflow)?
		.checked_mul(U256::from(997)).ok_or(ArithmeticError::Overflow)?;

	ensure!(!denominator.is_zero(), ArithmeticError::DivisionByZero);

	let amount_in = numerator
		.checked_div(denominator).ok_or(ArithmeticError::Overflow)?
		.checked_add(U256::one()).ok_or(ArithmeticError::Overflow)?;

	Ok(amount_in)
}


/// Calculate the amount of trading pairs of assets that can be obtained when remove liquidity.
/// Calculation formula:
/// - remove_liquidity / total_liquidity = remove_a / reserve_a
/// - remove_liquidity / total_liquidity = remove_b / reserve_b
pub fn calc_amount_out(
	reserve_a: U256,
	reserve_b: U256,
	remove_liquidity: U256,
	total_liquidity: U256,
) -> sp_std::result::Result<(U256, U256), ArithmeticError> {
	ensure!(!total_liquidity.is_zero(), ArithmeticError::DivisionByZero);

	let remove_amount_a = remove_liquidity
		.checked_mul(reserve_a).ok_or(ArithmeticError::Overflow)?
		.checked_div(total_liquidity).ok_or(ArithmeticError::Overflow)?;

	let remove_amount_b = remove_liquidity
		.checked_mul(reserve_b).ok_or(ArithmeticError::Overflow)?
		.checked_div(total_liquidity).ok_or(ArithmeticError::Overflow)?;

	Ok((remove_amount_a, remove_amount_b))
}


/// Calculate the amount of pairs of assets needed to add liquidity.
pub fn calc_amount_in(
	reserve_a: U256,
	reserve_b: U256,
	amount_a_desired: U256,
	amount_b_desired: U256,
	amount_a_min: U256,
	amount_b_min: U256,
) -> sp_std::result::Result<Option<(U256, U256)>, ArithmeticError> {
	if reserve_a == U256::zero() && reserve_b == U256::zero() {
		return Ok(Some((amount_a_desired, amount_b_desired)));
	}

	let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b)?;
	if amount_b_optimal <= amount_b_desired {
		if amount_b_optimal >= amount_b_min {
			Ok(Some((amount_a_desired, amount_b_optimal)))
		} else {
			Ok(None)
		}
	} else {
		let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a)?;
		if amount_a_optimal <= amount_a_desired && amount_a_optimal >= amount_a_min {
			Ok(Some((amount_a_optimal, amount_b_desired)))
		} else {
			Ok(None)
		}
	}
}

/// Calculate the amount of liquidity that can be obtained when adding a trading pair of assets.
/// Calculation formula:
/// - total_liquidity == 0 (first add)
///   add_liquidity = sqrt(amount_a * amount_b) - 1000
/// - total_liquidity != 0
///   add_liquidity = min(amount_a * total_liquidity / reserve_a,
///                       amount_b * total_liquidity / reserve_b)
pub fn calc_liquidity_add(
	reserve_a: U256,
	reserve_b: U256,
	amount_a: U256,
	amount_b: U256,
	total_liquidity: U256,
	minimum_liquidity_limit: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	if total_liquidity.is_zero() {
		let add_liquidity = amount_a
			.checked_mul(amount_b).ok_or(ArithmeticError::Overflow)?
			.integer_sqrt()
			.checked_sub(minimum_liquidity_limit).ok_or(ArithmeticError::Overflow)?;
		return Ok(add_liquidity);
	}

	ensure!(!reserve_a.is_zero(), ArithmeticError::DivisionByZero);
	ensure!(!reserve_b.is_zero(), ArithmeticError::DivisionByZero);

	let add_liquidity_a = amount_a
		.checked_mul(total_liquidity).ok_or(ArithmeticError::Overflow)?
		.checked_div(reserve_a).ok_or(ArithmeticError::Overflow)?;

	let add_liquidity_b = amount_b
		.checked_mul(total_liquidity).ok_or(ArithmeticError::Overflow)?
		.checked_div(reserve_b).ok_or(ArithmeticError::Overflow)?;

	let add_liquidity = if add_liquidity_a < add_liquidity_b { add_liquidity_a } else { add_liquidity_b };

	Ok(add_liquidity)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_quote() {
		assert_eq!(
			quote(U256::from(50), U256::from(100), U256::from(200)),
			Ok(U256::from(100))
		);
		assert_eq!(
			quote(U256::from(50), U256::from(0), U256::from(200)),
			Err(ArithmeticError::DivisionByZero)
		);
	}

	#[test]
	fn test_get_amount_out() {
		assert_eq!(
			get_amount_out(U256::from(50), U256::from(100), U256::from(200)),
			Ok(U256::from(66))
		);
		assert_eq!(
			get_amount_out(U256::from(50 * TEST_DECIMAL), U256::from(100 * TEST_DECIMAL), U256::from(200 * TEST_DECIMAL)),
			Ok(U256::from(665331998665u128))
		);
	}

	#[test]
	fn test_get_amount_in() {
		assert_eq!(
			get_amount_in(U256::from(66), U256::from(100), U256::from(200)),
			Ok(U256::from(50))
		);
		assert_eq!(
			get_amount_in(U256::from(66 * TEST_DECIMAL), U256::from(100 * TEST_DECIMAL), U256::from(200 * TEST_DECIMAL)),
			Ok(U256::from(494019371548u128))
		);
	}

	#[test]
	fn test_calc_amount_out() {
		assert_eq!(
			calc_amount_out(U256::from(50), U256::from(100), U256::from(100), U256::from(200)),
			Ok((U256::from(25), U256::from(50)))
		);
		assert_eq!(
			calc_amount_out(U256::from(50), U256::from(100), U256::from(100), U256::from(0)),
			Err(ArithmeticError::DivisionByZero)
		);
		assert_eq!(
			calc_amount_out(U256::from(U256::MAX/2), U256::from(U256::MAX), U256::from(U256::MAX/2), U256::from(U256::MAX)),
			Err(ArithmeticError::Overflow)
		);
	}

	#[test]
	fn test_calc_amount_in() {
		assert_eq!(
			calc_amount_in(U256::from(0), U256::from(0), U256::from(100), U256::from(200), U256::zero(), U256::zero()),
			Ok(Some((U256::from(100), U256::from(200))))
		);
		assert_eq!(
			calc_amount_in(U256::from(1000), U256::from(2000), U256::from(100), U256::from(500), U256::zero(), U256::zero()),
			Ok(Some((U256::from(100), U256::from(200))))
		);
		assert_eq!(
			calc_amount_in(U256::from(1000), U256::from(2000), U256::from(100), U256::from(500), U256::from(50), U256::from(250)),
			Ok(None)
		);
		assert_eq!(
			calc_amount_in(U256::from(1000), U256::from(2000), U256::from(100), U256::from(50), U256::from(0), U256::from(0)),
			Ok(Some((U256::from(25), U256::from(50))))
		);
		assert_eq!(
			calc_amount_in(U256::from(1000), U256::from(2000), U256::from(100), U256::from(50), U256::from(50), U256::from(0)),
			Ok(None)
		);
	}

	#[test]
	fn test_calc_liquidity_add() {
		assert_eq!(
			calc_liquidity_add(U256::from(0), U256::from(0), U256::from(10000), U256::from(40000), U256::from(0), U256::from(1000)),
			Ok(U256::from(19000))
		);
		assert_eq!(
			calc_liquidity_add(U256::from(10000), U256::from(40000), U256::from(10000), U256::from(40000), U256::from(19000), U256::from(1000)),
			Ok(U256::from(19000))
		);
	}
}
