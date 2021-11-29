#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use dico_primitives::{
	constants::time::{DAYS, MINUTES},
	BlockNumber,
};
use sp_core::U256;
use sp_runtime::ArithmeticError;

pub const BONE: u128 = 1_000_000_000_000_000_000u128;
pub const MIN_BPOW_BASE: u128 = 1u128;
pub const MAX_BPOW_BASE: u128 = 2u128 * BONE - 1u128;
pub const BPOW_PRECISION: u128 = BONE / 10_000_000_000u128;
pub const MAX_DURATION_BLOCK: BlockNumber = 3 * DAYS; // 43200 blocks if secs_per_block = 6
pub const MIN_DURATION_BLOCK: BlockNumber = 30 * MINUTES; // 300 blocks if secs_per_block = 6
pub const MAX_STEPS: u32 = MAX_DURATION_BLOCK / 25; // 432
pub const MIN_STEPS: u32 = MIN_DURATION_BLOCK / 25; // 3
pub const WEIGHT_ONE: u128 = 10_000_000_000u128;
pub const MAX_WEIGHT: u128 = 100 * WEIGHT_ONE;
pub const MIN_WEIGHT: u128 = WEIGHT_ONE;
pub const SWAP_FEE: u128 = 0u128;
pub const CROWDFUNDING_FEE: u128 = 5000000000000000u128; // 0.5% * BONE

#[cfg(test)]
const TEST_SWAP_FEE: u128 = 1_500_000_000u128;

fn btoi(a: U256) -> Option<U256> {
	a.checked_div(U256::from(BONE))
}

fn bfloor(a: U256) -> Option<U256> {
	btoi(a).and_then(|x| x.checked_mul(U256::from(BONE)))
}

pub fn badd(a: U256, b: U256) -> Option<U256> {
	a.checked_add(b)
}

pub fn bsub(a: U256, b: U256) -> Option<U256> {
	a.checked_sub(b)
}

pub fn bmul(a: U256, b: U256) -> Option<U256> {
	a.checked_mul(b)
		.and_then(|x| x.checked_add(U256::from(BONE).checked_div(U256::from(2))?))
		.and_then(|x| x.checked_div(U256::from(BONE)))
}

pub fn bdiv(a: U256, b: U256) -> Option<U256> {
	a.checked_mul(U256::from(BONE))
		.and_then(|x| x.checked_add(b.checked_div(U256::from(2))?))
		.and_then(|x| x.checked_div(b))
}

fn bpowi(a: U256, n: U256) -> Option<U256> {
	let zero = U256::zero();
	let two = U256::from(2);

	let mut z = if n.checked_rem(two)? != zero {
		a
	} else {
		U256::from(BONE)
	};

	let mut a1 = a;
	let mut n1 = n.checked_div(two)?;
	while n1 != zero {
		a1 = bmul(a1, a1)?;

		if n1.checked_rem(two)? != zero {
			z = bmul(z, a1)?;
		}

		n1 = n1.checked_div(two)?;
	}

	Some(z)
}

// Compute b^(e.w) by splitting it into (b^e)*(b^0.w).
// Use `bpowi` for `b^e` and `bpowK` for k iterations
// of approximation of b^0.w
fn bpow(base: U256, exp: U256) -> Option<U256> {
	if base < U256::from(MIN_BPOW_BASE) {
		return None;
	}
	if base > U256::from(MAX_BPOW_BASE) {
		return None;
	}

	let whole = bfloor(exp)?;
	let remain = bsub(exp, whole)?;

	let whole_pow = bpowi(base, btoi(whole)?)?;

	if remain == U256::zero() {
		return Some(whole_pow);
	}

	let partial_result = bpow_approx(base, remain, U256::from(BPOW_PRECISION))?;

	bmul(whole_pow, partial_result)
}

fn bsub_sign(a: U256, b: U256) -> (U256, bool) {
	if a >= b {
		(a - b, false)
	} else {
		(b - a, true)
	}
}

fn bpow_approx(base: U256, exp: U256, precision: U256) -> Option<U256> {
	let bone = U256::from(BONE);
	let zero = U256::zero();

	let a = exp;
	let (x, xneg) = bsub_sign(base, bone);
	let mut term = bone;
	let mut sum = term;
	let mut negative = false;

	// term(k) = numer / denom
	//         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
	// each iteration, multiply previous term by (a-(k-1)) * x / k
	// continue until term is less than precision
	let mut i = 1;
	while term >= precision {
		let big_k = U256::from(i).checked_mul(bone)?;
		let (c, cneg) = bsub_sign(a, bsub(big_k, bone)?);
		term = bmul(term, bmul(c, x)?)?;
		term = bdiv(term, big_k)?;
		if term == zero {
			break;
		}

		if xneg {
			negative = !negative
		}
		if cneg {
			negative = !negative
		}

		sum = if negative { bsub(sum, term)? } else { badd(sum, term)? };

		i += 1;
	}

	Some(sum)
}

// *******************************************************************************************//
//  calc_spot_price                                                                           //
//  sP = spot_price                                                                           //
//  bI = asset_balance_in              ( bI / wI )         1                                  //
//  bO = asset_balance_out       sP =  -----------  *  ----------                             //
//  wI = asset_weight_in               ( bO / wO )     ( 1 - sF )                             //
//  wO = asset_weight_out                                                                     //
//  sF = swap_fee                                                                             //
// *******************************************************************************************//
pub fn calc_spot_price(
	asset_balance_in: U256,
	asset_weight_in: U256,
	asset_balance_out: U256,
	asset_weight_out: U256,
	swap_fee: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	let bone = U256::from(BONE);

	let numerator = bdiv(asset_balance_in, asset_weight_in).ok_or(ArithmeticError::Overflow)?;
	let denominator = bdiv(asset_balance_out, asset_weight_out).ok_or(ArithmeticError::Overflow)?;
	let ratio = bdiv(numerator, denominator).ok_or(ArithmeticError::Overflow)?;
	let scale = bdiv(bone, bsub(bone, swap_fee).ok_or(ArithmeticError::Overflow)?).ok_or(ArithmeticError::Overflow)?;

	let spot_price = bmul(ratio, scale).ok_or(ArithmeticError::Overflow)?;
	Ok(spot_price)
}

//********************************************************************************************//
// calc_out_given_in                                                                          //
// aO = asset_amount_out                                                                      //
// bO = asset_balance_out                                                                     //
// bI = asset_balance_in               /      /            bI             \    (wI / wO) \    //
// aI = asset_amount_in     aO = bO * |  1 - | --------------------------  | ^            |   //
// wI = asset_weight_in                \      \ ( bI + ( aI * ( 1 - sF )) /              /    //
// wO = asset_weight_out                                                                      //
// sF = swap_fee                                                                              //
//********************************************************************************************//
pub fn calc_out_given_in(
	asset_balance_in: U256,
	asset_weight_in: U256,
	asset_balance_out: U256,
	asset_weight_out: U256,
	asset_amount_in: U256,
	swap_fee: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	let bone = U256::from(BONE);

	let weight_ratio = bdiv(asset_weight_in, asset_weight_out).ok_or(ArithmeticError::Overflow)?;
	let mut adjusted_in = bsub(bone, swap_fee).ok_or(ArithmeticError::Overflow)?;
	adjusted_in = bmul(asset_amount_in, adjusted_in).ok_or(ArithmeticError::Overflow)?;
	let x = badd(asset_balance_in, adjusted_in).ok_or(ArithmeticError::Overflow)?;
	let y = bdiv(asset_balance_in, x).ok_or(ArithmeticError::Overflow)?;
	let foo = bpow(y, weight_ratio).ok_or(ArithmeticError::Overflow)?;
	let bar = bsub(bone, foo).ok_or(ArithmeticError::Overflow)?;

	let asset_amount_out = bmul(asset_balance_out, bar).ok_or(ArithmeticError::Overflow)?;
	Ok(asset_amount_out)
}

//********************************************************************************************//
// calc_in_given_out                                                                          //
// aI = asset_amount_in                                                                       //
// bO = asset_balance_out                  /  /     bO      \    (wO / wI)      \             //
// bI = asset_balance_in             bI * |  | ------------  | ^            - 1  |            //
// aO = asset_amount_out       aI =        \  \ ( bO - aO ) /                   /             //
// wI = asset_weight_in              --------------------------------------------             //
// wO = asset_weight_out                             ( 1 - sF )                               //
// sF = swap_fee                                                                              //
//********************************************************************************************//
pub fn calc_in_given_out(
	asset_balance_in: U256,
	asset_weight_in: U256,
	asset_balance_out: U256,
	asset_weight_out: U256,
	asset_amount_out: U256,
	swap_fee: U256,
) -> sp_std::result::Result<U256, ArithmeticError> {
	let bone = U256::from(BONE);

	let weight_radio = bdiv(asset_weight_out, asset_weight_in).ok_or(ArithmeticError::Overflow)?;
	let diff = bsub(asset_balance_out, asset_amount_out).ok_or(ArithmeticError::Overflow)?;
	let y = bdiv(asset_balance_out, diff).ok_or(ArithmeticError::Overflow)?;
	let mut foo = bpow(y, weight_radio).ok_or(ArithmeticError::Overflow)?;
	foo = bsub(foo, bone).ok_or(ArithmeticError::Overflow)?;
	let mut asset_amount_in = bsub(bone, swap_fee).ok_or(ArithmeticError::Overflow)?;
	asset_amount_in = bdiv(
		bmul(asset_balance_in, foo).ok_or(ArithmeticError::Overflow)?,
		asset_amount_in,
	)
	.ok_or(ArithmeticError::Overflow)?;

	Ok(asset_amount_in)
}

//********************************************************************************************//
// calc_adjust_block                                                                          //
// sB = start_block                                                                           //
// eB = end_block                     eB - sB                                                 //
// sS = steps                aB = ---------------  * sT + sB                                  //
// sT = step                            sS                                                    //
// aB = adjust_block                                                                          //
//********************************************************************************************//
pub fn calc_adjust_block(start_block: U256, end_block: U256, steps: U256, step: U256) -> Option<U256> {
	if steps == U256::zero() {
		return None;
	}

	bsub(end_block, start_block)
		.and_then(|x| bdiv(x, steps))
		.and_then(|x| bmul(x, step))
		.and_then(|x| badd(x, start_block))
}

//************************************************************************************************//
// calc_adjust_weight                                                                             //
// sW = start_weight                                                                              //
// eW = end_weight            eW - sW                                 sW - eW                     //
// sS = steps      aW = sW + --------- * sT (eW > sW)  or  aW = sW - ---------- * sT (eW < sW)    //
// sT = step                    sS                                       sS                       //
// aW = adjust_weight                                                                             //
//************************************************************************************************//
pub fn calc_adjust_weight(start_weight: U256, end_weight: U256, steps: U256, step: U256) -> Option<U256> {
	if steps == U256::zero() {
		return None;
	}

	if start_weight > end_weight {
		let acc_weight = bsub(start_weight, end_weight)
			.and_then(|x| bdiv(x, steps))
			.and_then(|x| bmul(x, step))?;
		bsub(start_weight, acc_weight)
	} else if start_weight < end_weight {
		let acc_weight = bsub(end_weight, start_weight)
			.and_then(|x| bdiv(x, steps))
			.and_then(|x| bmul(x, step))?;
		badd(start_weight, acc_weight)
	} else {
		Some(start_weight)
	}
}

pub fn calc_crowdfunding_amount(supply_amount: U256, crowdfunding_fee: U256) -> Option<U256> {
	bsub(U256::from(BONE), U256::from(crowdfunding_fee)).and_then(|x| bmul(U256::from(supply_amount), x))
}

pub fn calc_supply_amount_with_fee(supply_amount: U256, crowdfunding_fee: U256) -> Option<U256> {
	bsub(U256::from(BONE), U256::from(crowdfunding_fee)).and_then(|x| bdiv(U256::from(supply_amount), x))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_calc_supply_amount_with_fee() {
		assert_eq!(
			calc_supply_amount_with_fee(U256::from(10000000000000u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(10050251256281u128)),
		);
		assert_eq!(
			calc_crowdfunding_amount(U256::from(10050251256281u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(10000000000000u128)),
		);
		assert_eq!(
			calc_supply_amount_with_fee(U256::from(86033999974477294587667u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(86466331632640497073032u128)),
		);
	}

	#[test]
	fn test_calc_crowdfunding_fee() {
		assert_eq!(
			calc_crowdfunding_amount(U256::from(10000000000000u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(9950000000000u128)),
		);
		assert_eq!(
			calc_crowdfunding_amount(U256::from(12356789876540u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(12295005927157u128)),
		);
		assert_eq!(
			calc_crowdfunding_amount(U256::from(86034000000000000000000u128), U256::from(CROWDFUNDING_FEE)),
			Some(U256::from(85603830000000000000000u128))
		);
	}

	#[test]
	fn test_calc_adjust_weight() {
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(0)
			),
			Some(U256::from(36000000000000u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(1)
			),
			Some(U256::from(35666666666667u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(4)
			),
			Some(U256::from(34666666666667u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(14)
			),
			Some(U256::from(31333333333333u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(34)
			),
			Some(U256::from(24666666666667u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(64)
			),
			Some(U256::from(14666666666667u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(36000000000000u128),
				U256::from(12000000000000u128),
				U256::from(72),
				U256::from(72)
			),
			Some(U256::from(12000000000000u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(4000000000000u128),
				U256::from(28000000000000u128),
				U256::from(72),
				U256::from(4)
			),
			Some(U256::from(5333333333333u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(4000000000000u128),
				U256::from(28000000000000u128),
				U256::from(72),
				U256::from(14)
			),
			Some(U256::from(8666666666667u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(4000000000000u128),
				U256::from(28000000000000u128),
				U256::from(72),
				U256::from(34)
			),
			Some(U256::from(15333333333333u128)),
		);
		assert_eq!(
			calc_adjust_weight(
				U256::from(4000000000000u128),
				U256::from(28000000000000u128),
				U256::from(72),
				U256::from(64)
			),
			Some(U256::from(25333333333333u128)),
		);
	}

	#[test]
	fn test_calc_adjust_block() {
		assert_eq!(
			calc_adjust_block(U256::from(10), U256::from(19200), U256::from(72), U256::from(0)),
			Some(U256::from(10)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(1)),
			Some(U256::from(267)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(20)),
			Some(U256::from(5333)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(32)),
			Some(U256::from(8533)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(41)),
			Some(U256::from(10933)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(55)),
			Some(U256::from(14667)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(70)),
			Some(U256::from(18667)),
		);
		assert_eq!(
			calc_adjust_block(U256::from(0), U256::from(19200), U256::from(72), U256::from(72)),
			Some(U256::from(19200)),
		);
	}

	#[test]
	fn test_btoi() {
		assert_eq!(btoi(U256::zero()), Some(U256::zero()));
		assert_eq!(btoi(U256::from(100000000000000000000u128)), Some(U256::from(100)));
		assert_eq!(
			btoi(U256::MAX),
			Some(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457").unwrap())
		);
	}

	#[test]
	fn test_bfloor() {
		assert_eq!(bfloor(U256::zero()), Some(U256::zero()));
		assert_eq!(
			bfloor(U256::from(100000000000000000000u128)),
			Some(U256::from(100000000000000000000u128))
		);
		assert_eq!(
			bfloor(U256::MAX),
			Some(
				U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457000000000000000000")
					.unwrap()
			)
		);
	}

	#[test]
	fn test_badd() {
		assert_eq!(badd(U256::zero(), U256::one()), Some(U256::one()));
		assert_eq!(
			badd(U256::from(100000000000000000000u128), U256::one()),
			Some(U256::from(100000000000000000001u128))
		);
		assert_eq!(badd(U256::MAX - U256::from(3), U256::from(3)), Some(U256::MAX));
	}

	#[test]
	fn test_bsub() {
		assert_eq!(bsub(U256::one(), U256::zero()), Some(U256::one()));
		assert_eq!(bsub(U256::one(), U256::one()), Some(U256::zero()));
		assert_eq!(
			bsub(U256::from(100000000000000000000u128), U256::one()),
			Some(U256::from(99999999999999999999u128))
		);
		assert_eq!(
			bsub(U256::MAX, U256::from(100000000000000000000u128)),
			Some(
				U256::from_dec_str("115792089237316195423570985008687907853269984665640564039357584007913129639935")
					.unwrap()
			)
		);
	}

	#[test]
	fn test_bmul() {
		assert_eq!(bmul(U256::zero(), U256::one()), Some(U256::zero()));
		assert_eq!(bmul(U256::one(), U256::one()), Some(U256::zero()));
		assert_eq!(
			bmul(U256::from(100000000000000000000u128), U256::one()),
			Some(U256::from(100u128))
		);
		assert_eq!(
			bmul(U256::from(u128::MAX), U256::from(u128::MAX)),
			Some(U256::from_dec_str("115792089237316195423570985008687907852589419931798687112531").unwrap())
		);
	}

	#[test]
	fn test_bdiv() {
		assert_eq!(bdiv(U256::zero(), U256::one()), Some(U256::zero()));
		assert_eq!(
			bdiv(U256::one(), U256::one()),
			Some(U256::from(1000000000000000000u128))
		);
		assert_eq!(
			bdiv(U256::from(100000000000000000000u128), U256::from(30000000u128)),
			Some(U256::from(3333333333333333333333333333333u128))
		);
		assert_eq!(
			bdiv(U256::from(BONE), U256::from(BONE)),
			Some(U256::from(1000000000000000000u128))
		);
	}

	#[test]
	fn test_bpowi() {
		assert_eq!(
			bpowi(U256::zero(), U256::zero()),
			Some(U256::from(1000000000000000000u128))
		);
		assert_eq!(bpowi(U256::one(), U256::one()), Some(U256::one()));
		assert_eq!(
			bpowi(U256::from(10000000u128), U256::from(30000000u128)),
			Some(U256::zero())
		);
		assert_eq!(
			bpowi(U256::from(BONE), U256::from(BONE)),
			Some(U256::from(1000000000000000000u128))
		);
		assert_eq!(
			bpowi(U256::from(1000000000000000000000000000u128), U256::from(30000000u128)),
			None
		);
	}

	#[test]
	fn test_bpow() {
		assert_eq!(bpow(U256::zero(), U256::zero()), None);
		assert_eq!(bpow(U256::one(), U256::one()), Some(U256::from(999999999999999999u128)));
		assert_eq!(
			bpow(U256::from(10000000u128), U256::from(30000000u128)),
			Some(U256::from(999999999970000000u128))
		);
		assert_eq!(
			bpow(U256::from(BONE), U256::from(BONE)),
			Some(U256::from(1000000000000000000u128))
		);
		assert_eq!(
			bpow(U256::from(1000000000000000000000000000u128), U256::from(30000000u128)),
			None
		);
	}

	#[test]
	fn test_calc_spot_price() {
		assert_eq!(
			calc_spot_price(
				U256::from(10000000000000u128),
				U256::from(4),
				U256::from(75000000000000000u128),
				U256::from(36),
				U256::zero(),
			),
			Ok(U256::from(1200000000000000u128))
		);
		assert_eq!(
			calc_spot_price(
				U256::zero(),
				U256::from(4),
				U256::from(75000000000000000u128),
				U256::from(36),
				U256::zero(),
			),
			Ok(U256::zero())
		);
		assert_eq!(
			calc_spot_price(
				U256::from(10000000000000u128),
				U256::from(4),
				U256::from(75000000000000000u128),
				U256::from(36),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(1200000001800000u128))
		);

		assert_eq!(
			calc_spot_price(
				U256::from(u128::MAX),
				U256::from(35),
				U256::from(u128::MAX),
				U256::from(5),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(142857143071428572u128))
		);

		assert_eq!(
			calc_spot_price(
				U256::from(1333333000000000000000000u128),
				U256::from(4),
				U256::from(7500000000000000000000000u128),
				U256::from(36),
				U256::from(1500000000000000u128),
			),
			Ok(U256::from(1602403204807210816u128))
		);
	}

	#[test]
	fn test_calc_out_given_in() {
		assert_eq!(
			calc_out_given_in(
				U256::from(u128::MAX),
				U256::from(35),
				U256::from(u128::MAX),
				U256::from(5),
				U256::from(10000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::zero())
		);
		assert_eq!(
			calc_out_given_in(
				U256::from(10000000000000u128),
				U256::from(4),
				U256::from(75000000000000000u128),
				U256::from(36),
				U256::from(1000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(790060733782324u128))
		);
		assert_eq!(
			calc_out_given_in(
				U256::from(10000000000000u128),
				U256::from(36),
				U256::from(75000000000000000u128),
				U256::from(4),
				U256::from(1000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(43192678583027373u128))
		);
		assert_eq!(
			calc_out_given_in(
				U256::from(1333333000000u128),
				U256::from(4),
				U256::from(7500000000000000000u128),
				U256::from(36),
				U256::from(86034000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(51927050546147730u128))
		);
	}

	#[test]
	fn test_calc_in_given_out() {
		assert_eq!(
			calc_in_given_out(
				U256::from(u128::MAX),
				U256::from(35),
				U256::from(u128::MAX),
				U256::from(5),
				U256::from(10000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::zero())
		);
		assert_eq!(
			calc_in_given_out(
				U256::from(10000000000000u128),
				U256::from(4),
				U256::from(75000000000000000u128),
				U256::from(36),
				U256::from(1000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(1200080006u128))
		);
		assert_eq!(
			calc_in_given_out(
				U256::from(10000000000000u128),
				U256::from(36),
				U256::from(75000000000000000u128),
				U256::from(4),
				U256::from(1000000000000u128),
				U256::from(TEST_SWAP_FEE),
			),
			Ok(U256::from(14814925u128))
		);
	}
}
