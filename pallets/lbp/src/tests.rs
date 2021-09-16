//! Unit tests for the lbp module.

#![cfg(test)]

use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, Origin,
	System, Test, DOT, ALICE, BOB, USDT, DICO, Lbp,
	DEFAULT_ASSET_AMOUNT, WEIGHT_ONE
};
use frame_support::{assert_ok, assert_err};

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
fn create_lbp_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			1000,
			100,
		));

		assert_eq!(NextLbpId::<Test>::get(), 1);

		let lbp_pair = LbpPair::new(USDT, DICO);
		assert_eq!(OngoingLbps::<Test>::get(lbp_pair), Some((ALICE, 0)));

		let lbp_info = LbpInfo::new(100, 1000, 100,ALICE,
									USDT, DICO, 10_000_000_000_000u128,
									100_000_000_000_000u128, 10 * WEIGHT_ONE, 90 * WEIGHT_ONE,
									90 * WEIGHT_ONE, 10 * WEIGHT_ONE);
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		let module_id_account = Lbp::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 100_000_000_000_000u128);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 10_000_000_000_000u128);
		assert_eq!(Currency::free_balance(DICO, &ALICE), DEFAULT_ASSET_AMOUNT - 100_000_000_000_000u128);
		assert_eq!(Currency::free_balance(USDT, &ALICE), DEFAULT_ASSET_AMOUNT - 10_000_000_000_000u128);

		expect_events(
			vec![Event::LbpCreated(ALICE, 0, USDT, DICO,
								   10_000_000_000_000, 100_000_000_000_000).into()]
		);

		assert_err!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			1000,
			100,
		), Error::<Test>::LbpPairOngoing);

		assert_err!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			1000,
			0,
		), Error::<Test>::ErrMinSteps);

		assert_err!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			1000,
			433,
		), Error::<Test>::ErrMaxSteps);

		assert_err!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			120,
			100,
		), Error::<Test>::ErrMinDurationBlock);

		assert_err!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			10_000_000_000_000u128,
			100_000_000_000_000u128,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			100,
			100000,
			100,
		), Error::<Test>::ErrMaxDurationBlock);
	});
}

#[test]
fn exit_lbp_should_work() {
	new_test_ext().execute_with(|| {
		// preset storage
		assert_ok!(Lbp::create_lbp(
			Origin::signed(ALICE),
			DICO,
			USDT,
			100_000_000_000_000u128,
			10_000_000_000_000u128,
			90 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			10 * WEIGHT_ONE,
			90 * WEIGHT_ONE,
			100,
			1000,
			100,
		));

		let mut lbp_info = LbpInfo::new(100, 1000, 100, ALICE,
										DICO, USDT, 100_000_000_000_000u128,
										10_000_000_000_000u128, 90 * WEIGHT_ONE, 10 * WEIGHT_ONE,
										10 * WEIGHT_ONE, 90 * WEIGHT_ONE);


		assert_err!(Lbp::exit_lbp(Origin::signed(ALICE), 1), Error::<Test>::LbpNotFind);
		assert_err!(Lbp::exit_lbp(Origin::signed(BOB), 0), Error::<Test>::MustBeOwner);

		lbp_info.status = LbpStatus::Cancelled;
		Lbps::<Test>::insert(0, lbp_info);
		assert_err!(Lbp::exit_lbp(Origin::signed(ALICE), 0), Error::<Test>::MustBeNonTradingStatus);

		lbp_info.status = LbpStatus::InProgress;
		Lbps::<Test>::insert(0, lbp_info);
		assert_err!(Lbp::exit_lbp(Origin::signed(ALICE), 0), Error::<Test>::MustBeNonTradingStatus);

		lbp_info.status = LbpStatus::Pending;
		Lbps::<Test>::insert(0, lbp_info);

		assert_ok!(Lbp::exit_lbp(Origin::signed(ALICE), 0));

		let module_id_account = Lbp::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 0);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 0);
		assert_eq!(Currency::free_balance(DICO, &ALICE), DEFAULT_ASSET_AMOUNT);
		assert_eq!(Currency::free_balance(USDT, &ALICE), DEFAULT_ASSET_AMOUNT);

		lbp_info.status = LbpStatus::Cancelled;
		lbp_info.afs_balance = 0;
		lbp_info.fundraising_balance = 0;
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		let lbp_pair = LbpPair::new(DICO, USDT);
		assert_eq!(OngoingLbps::<Test>::get(lbp_pair), None);

		expect_events(
			vec![Event::LbpExited(ALICE, 0).into()]
		);
	});
}

#[test]
fn swap_exact_amount_supply_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			1333333000000000000000000u128,
			7500000000000000000000000u128,
			4 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			4 * WEIGHT_ONE,
			1,
			1001,
			100,
		));

		assert_eq!(NextLbpId::<Test>::get(), 1);

		let lbp_pair = LbpPair::new(USDT, DICO);
		assert_eq!(OngoingLbps::<Test>::get(lbp_pair), Some((ALICE, 0)));

		let mut lbp_info = LbpInfo::new(1, 1001, 100, ALICE,
										USDT, DICO, 1333333000000000000000000u128,
										7500000000000000000000000u128, 4 * WEIGHT_ONE, 36 * WEIGHT_ONE,
										36 * WEIGHT_ONE, 4 * WEIGHT_ONE);
		lbp_info.status = LbpStatus::InProgress;
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		let module_id_account = Lbp::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 7500000000000000000000000u128);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 1333333000000000000000000u128);
		assert_eq!(Currency::free_balance(DICO, &ALICE), DEFAULT_ASSET_AMOUNT - 7500000000000000000000000u128);
		assert_eq!(Currency::free_balance(USDT, &ALICE), DEFAULT_ASSET_AMOUNT - 1333333000000000000000000u128);

		expect_events(
			vec![Event::LbpCreated(ALICE, 0, USDT, DICO,
								   1333333000000000000000000u128, 7500000000000000000000000u128).into()]
		);

		assert_ok!(Lbp::swap_exact_amount_supply(
			Origin::signed(BOB),
			USDT,
			86034000000000000000000u128,
			DICO,
			0
		));

		lbp_info.afs_balance += 86034000000000000000000u128;
		lbp_info.fundraising_balance -= 51927050621361330000000u128;
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		assert_eq!(Currency::free_balance(DICO, &module_id_account),
				   7500000000000000000000000u128 - 51927050621361330000000u128);
		assert_eq!(Currency::free_balance(USDT, &module_id_account),
				   1333333000000000000000000u128 + 86034000000000000000000u128);
		assert_eq!(Currency::free_balance(DICO, &BOB),
				   DEFAULT_ASSET_AMOUNT + 51927050621361330000000u128);
		assert_eq!(Currency::free_balance(USDT, &BOB),
				   DEFAULT_ASSET_AMOUNT - 86034000000000000000000u128);

		expect_events(
			vec![Event::Swapped(BOB, 0, USDT, DICO,
								86034000000000000000000u128, 51927050621361330000000u128).into()]
		);
	});
}


#[test]
fn swap_exact_amount_target_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Lbp::create_lbp(
			Origin::signed(ALICE),
			USDT,
			DICO,
			1333333000000000000000000u128,
			7500000000000000000000000u128,
			4 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			36 * WEIGHT_ONE,
			4 * WEIGHT_ONE,
			1,
			1001,
			100,
		));

		assert_eq!(NextLbpId::<Test>::get(), 1);

		let lbp_pair = LbpPair::new(USDT, DICO);
		assert_eq!(OngoingLbps::<Test>::get(lbp_pair), Some((ALICE, 0)));

		let mut lbp_info = LbpInfo::new(1, 1001, 100, ALICE,
										USDT, DICO, 1333333000000000000000000u128,
										7500000000000000000000000u128, 4 * WEIGHT_ONE, 36 * WEIGHT_ONE,
										36 * WEIGHT_ONE, 4 * WEIGHT_ONE);
		lbp_info.status = LbpStatus::InProgress;
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		let module_id_account = Lbp::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 7500000000000000000000000u128);
		assert_eq!(Currency::free_balance(USDT, &module_id_account), 1333333000000000000000000u128);
		assert_eq!(Currency::free_balance(DICO, &ALICE), DEFAULT_ASSET_AMOUNT - 7500000000000000000000000u128);
		assert_eq!(Currency::free_balance(USDT, &ALICE), DEFAULT_ASSET_AMOUNT - 1333333000000000000000000u128);

		expect_events(
			vec![Event::LbpCreated(ALICE, 0, USDT, DICO,
								   1333333000000000000000000u128, 7500000000000000000000000u128).into()]
		);

		assert_ok!(Lbp::swap_exact_amount_target(
			Origin::signed(BOB),
			USDT,
			986034000000000000000000u128,
			DICO,
			51927050621361330000000u128
		));

		lbp_info.afs_balance += 86033999974477294587667u128;
		lbp_info.fundraising_balance -= 51927050621361330000000u128;
		assert_eq!(Lbps::<Test>::get(0), Some(lbp_info));

		assert_eq!(Currency::free_balance(DICO, &module_id_account),
				   7500000000000000000000000u128 - 51927050621361330000000u128);
		assert_eq!(Currency::free_balance(USDT, &module_id_account),
				   1333333000000000000000000u128 + 86033999974477294587667u128);
		assert_eq!(Currency::free_balance(DICO, &BOB),
				   DEFAULT_ASSET_AMOUNT + 51927050621361330000000u128);
		assert_eq!(Currency::free_balance(USDT, &BOB),
				   DEFAULT_ASSET_AMOUNT - 86033999974477294587667u128);

		expect_events(
			vec![Event::Swapped(BOB, 0, USDT, DICO,
								86033999974477294587667u128, 51927050621361330000000u128).into()]
		);
	});
}


