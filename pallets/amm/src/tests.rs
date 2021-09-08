//! Unit tests for the amm module.

#![cfg(test)]

use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, Origin,
	System, Test, DOT, ALICE, BOB, USDT, DICO, AMM,
	DEFAULT_ASSET_AMOUNT,
};
use frame_support::{assert_ok};


pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn last_events(n: usize) -> Vec<TestEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<TestEvent>) {
	assert_eq!(last_events(e.len()), e);
}

#[test]
fn add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = DICO;
		let asset_b = USDT;
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		let module_id_account = AMM::account_id();
		let pair = AMM::pair_for(asset_a, asset_b);
		let liquidity_info = Liquidity::<Test>::get(pair).unwrap();
		let liquidity_id = liquidity_info.2;

		assert_eq!(Currency::free_balance(asset_a, &module_id_account), 100_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &module_id_account), 100_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_a, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);
		assert_eq!(Currency::free_balance(liquidity_id, &ALICE), 100_000_000_000_000 - 1000);
		assert_eq!(Currency::total_issuance(liquidity_id), 100_000_000_000_000);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(100_000_000_000_000, 100_000_000_000_000, 20000000)
		);

		expect_events(vec![Event::LiquidityAdded(ALICE, liquidity_id,
												 asset_a, asset_b,
												 100_000_000_000_000,
												 100_000_000_000_000).into()]);

		assert_ok!(AMM::add_liquidity(
			Origin::signed(BOB),
			asset_a,
			asset_b,
			200_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_eq!(Currency::free_balance(asset_a, &module_id_account), 200_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &module_id_account), 200_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_a, &BOB), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &BOB), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000);
		assert_eq!(Currency::free_balance(liquidity_id, &BOB), 100_000_000_000_000);
		assert_eq!(Currency::total_issuance(liquidity_id), 200_000_000_000_000);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(200_000_000_000_000, 200_000_000_000_000, 20000000)
		);

		expect_events(vec![Event::LiquidityAdded(BOB, liquidity_id,
												 asset_a, asset_b,
												 100_000_000_000_000,
												 100_000_000_000_000).into()]);
	});
}

#[test]
fn remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = DICO;
		let asset_b = USDT;
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::remove_liquidity(
			Origin::signed(ALICE),
			asset_a,
			asset_b,
			100_000_000_000_000 - 1000,
			0,
			0
		));

		let module_id_account = AMM::account_id();
		let pair = AMM::pair_for(asset_a, asset_b);
		let liquidity_info = Liquidity::<Test>::get(pair).unwrap();
		let liquidity_id = liquidity_info.2;

		assert_eq!(Currency::free_balance(asset_a, &module_id_account), 1000);
		assert_eq!(Currency::free_balance(asset_b, &module_id_account), 1000);
		assert_eq!(Currency::free_balance(asset_a, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000 + 99999999999000);
		assert_eq!(Currency::free_balance(asset_b, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000 + 99999999999000);
		assert_eq!(Currency::free_balance(liquidity_id, &ALICE), 0);
		assert_eq!(Currency::total_issuance(liquidity_id), 1000);

		let pair = AMM::pair_for(asset_a, asset_b);
		assert_eq!(Liquidity::<Test>::get(pair).unwrap(), LiquidityInfo(1000, 1000, 20000000));

		expect_events(vec![Event::LiquidityRemoved(ALICE, liquidity_id,
												   asset_a, asset_b,
												   100_000_000_000_000 - 1000).into()]);
	});
}


#[test]
fn swap_exact_assets_for_assets_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = DICO;
		let asset_b = USDT;
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::swap_exact_assets_for_assets(
			Origin::signed(BOB),
			10_000_000_000_000,
			0,
			vec![DICO, USDT]
		));

		// path: [DICO, USDT]
		// amounts: [10_000_000_000_000, 9066108938801]

		let module_id_account = AMM::account_id();

		assert_eq!(Currency::free_balance(asset_a, &module_id_account), 110_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &module_id_account), 100_000_000_000_000 - 9066108938801);
		assert_eq!(Currency::free_balance(asset_a, &BOB), DEFAULT_ASSET_AMOUNT - 10_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_b, &BOB), DEFAULT_ASSET_AMOUNT + 9066108938801);

		let pair = AMM::pair_for(asset_a, asset_b);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(110_000_000_000_000, 100_000_000_000_000 - 9066108938801, 20000000)
		);

		expect_events(vec![Event::Swapped(BOB, vec![DICO, USDT],
										  10_000_000_000_000, 9066108938801).into()]);
	});
}

#[test]
fn swap_exact_assets_for_assets_2_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			DICO,
			USDT,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			DOT,
			USDT,
			100_000_000_000_000,
			500_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::swap_exact_assets_for_assets(
			Origin::signed(BOB),
			10_000_000_000_000,
			0,
			vec![DICO, USDT, DOT]
		));

		let module_id_account = AMM::account_id();

		// path: [DICO, USDT, DOT]
		// amounts: [10_000_000_000_000, 9066108938801, 1775681666676]

		assert_eq!(Currency::free_balance(DICO, &module_id_account), 110_000_000_000_000);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 600_000_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &module_id_account), 100_000_000_000_000 - 1775681666676);
		assert_eq!(Currency::free_balance(DICO, &BOB), DEFAULT_ASSET_AMOUNT - 10_000_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &BOB), DEFAULT_ASSET_AMOUNT + 1775681666676);

		let pair = AMM::pair_for(DICO, USDT);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(110_000_000_000_000, 100_000_000_000_000 - 9066108938801, 20000000)
		);

		let pair = AMM::pair_for(DOT, USDT);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(100_000_000_000_000-1775681666676, 500_000_000_000_000 + 9066108938801, 20000001)
		);


		expect_events(vec![Event::Swapped(BOB, vec![DICO, USDT, DOT],
										  10_000_000_000_000, 1775681666676).into()]);
	});
}

#[test]
fn swap_assets_for_exact_assets_should_work() {
	new_test_ext().execute_with(|| {
		let asset_a = DICO;
		let asset_b = USDT;
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			asset_a,
			asset_b,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::swap_assets_for_exact_assets(
			Origin::signed(BOB),
			10_000_000_000_000,
			100_000_000_000_000,
			vec![DICO, USDT]
		));

		// path: [DICO, USDT]
		// amounts: [11144544745348, 10_000_000_000_000]

		let module_id_account = AMM::account_id();

		assert_eq!(Currency::free_balance(asset_a, &module_id_account), 100_000_000_000_000 + 11144544745348);
		assert_eq!(Currency::free_balance(asset_b, &module_id_account), 100_000_000_000_000 - 10_000_000_000_000);
		assert_eq!(Currency::free_balance(asset_a, &BOB), DEFAULT_ASSET_AMOUNT - 11144544745348);
		assert_eq!(Currency::free_balance(asset_b, &BOB), DEFAULT_ASSET_AMOUNT + 10_000_000_000_000);

		let pair = AMM::pair_for(asset_a, asset_b);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(100_000_000_000_000 + 11144544745348, 100_000_000_000_000 - 10_000_000_000_000, 20000000)
		);

		expect_events(vec![Event::Swapped(BOB, vec![DICO, USDT],
										  11144544745348, 10_000_000_000_000).into()]);
	});
}

#[test]
fn swap_assets_for_exact_assets2_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			DICO,
			USDT,
			100_000_000_000_000,
			100_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::add_liquidity(
			Origin::signed(ALICE),
			DOT,
			USDT,
			100_000_000_000_000,
			500_000_000_000_000,
			0,
			0
		));

		assert_ok!(AMM::swap_assets_for_exact_assets(
			Origin::signed(BOB),
			1775681666676,
			100_000_000_000_000,
			vec![DICO, USDT, DOT]
		));

		let module_id_account = AMM::account_id();

		// path: [DICO, USDT, DOT]
		// amounts: [10_000_000_000_000, 9066108938801, 1775681666676]

		assert_eq!(Currency::free_balance(DICO, &module_id_account), 110_000_000_000_000);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 600_000_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &module_id_account), 100_000_000_000_000 - 1775681666676);
		assert_eq!(Currency::free_balance(DICO, &BOB), DEFAULT_ASSET_AMOUNT - 10_000_000_000_000);
		assert_eq!(Currency::free_balance(DOT, &BOB), DEFAULT_ASSET_AMOUNT + 1775681666676);

		let pair = AMM::pair_for(DICO, USDT);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(110_000_000_000_000, 100_000_000_000_000 - 9066108938801, 20000000)
		);

		let pair = AMM::pair_for(DOT, USDT);
		assert_eq!(
			Liquidity::<Test>::get(pair).unwrap(),
			LiquidityInfo(100_000_000_000_000-1775681666676, 500_000_000_000_000 + 9066108938801, 20000001)
		);

		expect_events(vec![Event::Swapped(BOB, vec![DICO, USDT, DOT],
										  10_000_000_000_000, 1775681666676).into()]);
	});
}

