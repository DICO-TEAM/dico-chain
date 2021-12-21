//! Unit tests for the farm-extend module.

#![cfg(test)]

use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, FarmExtend, Origin, System, Test, ALICE, BOB, DEFAULT_ASSET_AMOUNT, DICO,
	DOT, USDT,
};
use frame_support::assert_ok;

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
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(FarmExtend::create_pool(
			Origin::signed(ALICE),
			DOT,
			100,
			1100,
			1_000_000_000,
			DICO
		));

		let currency_amount: Balance = (1100 - 100) * 1_000_000_000;
		let pool_extend_info = PoolExtendInfo::new(DOT, currency_amount, ALICE, 100, 1100, 1_000_000_000, 100, DICO);
		assert_eq!(NextPoolExtendId::<Test>::get(), 1);
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		expect_events(vec![
			Event::PoolExtendCreated(ALICE, 0, DOT, currency_amount, DICO).into()
		]);

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DOT, &module_id_account), currency_amount);
		assert_eq!(
			Currency::free_balance(DOT, &ALICE),
			DEFAULT_ASSET_AMOUNT - currency_amount
		);
	});
}

#[test]
fn deposit_asset_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(FarmExtend::create_pool(
			Origin::signed(ALICE),
			DOT,
			100,
			1100,
			1_000_000_000,
			DICO
		));

		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		let currency_amount: Balance = (1100 - 100) * 1_000_000_000;
		let mut pool_extend_info =
			PoolExtendInfo::new(DOT, currency_amount, ALICE, 100, 1100, 1_000_000_000, 100, DICO);
		pool_extend_info.total_stake_amount = 1000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 1000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &ALICE),
			DEFAULT_ASSET_AMOUNT - 1000_000_000_000
		);
		expect_events(vec![Event::AssetDeposited(ALICE, 0, 1000_000_000_000).into()]);

		System::set_block_number(100);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 2000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 2000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &BOB),
			DEFAULT_ASSET_AMOUNT - 1000_000_000_000
		);

		System::set_block_number(200);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 3000_000_000_000;
		pool_extend_info.last_reward_block = 200;
		// acc_reward_per_share = (200 - 100) * 1000000000 * 1e12 / 2000000000000 = 50000000000
		pool_extend_info.acc_reward_per_share = 50000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));
		// reward_debt = 2000000000000 * 50000000000 / 1e12 = 100000000000
		let participant_extend = ParticipantExtend::new(2000_000_000_000, 100000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 3000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &BOB),
			DEFAULT_ASSET_AMOUNT - 2000_000_000_000
		);

		System::set_block_number(2000);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 4000_000_000_000;
		pool_extend_info.last_reward_block = 1100;
		// add_reward_per_share = (1100 - 200) * 1000000000 * 1e12 / 3000000000000 = 300000000000
		// acc_reward_per_share = 50000000000 + 300000000000 = 350000000000
		pool_extend_info.acc_reward_per_share = 350000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		// reward_debt = 2000000000000 * 350000000000 / 1e12 = 700000000000
		let participant_extend = ParticipantExtend::new(2000_000_000_000, 700000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));

		System::set_block_number(3000);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 5000_000_000_000;
		pool_extend_info.last_reward_block = 1100;
		// add_reward_per_share = (1100 - 1100) * 1000000000 * 1e12 / 3000000000000 = 0
		// acc_reward_per_share = 0 + 350000000000 = 350000000000
		pool_extend_info.acc_reward_per_share = 350000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		// reward_debt = 3000000000000 * 350000000000 / 1e12 = 1050000000000
		let participant_extend = ParticipantExtend::new(3000_000_000_000, 1050000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		assert_eq!(Currency::free_balance(DOT, &module_id_account), 0);
		assert_eq!(
			Currency::free_balance(DOT, &ALICE),
			DEFAULT_ASSET_AMOUNT - currency_amount + 350000000000
		);
		assert_eq!(
			Currency::free_balance(DOT, &BOB),
			DEFAULT_ASSET_AMOUNT + 50000000000 + 600000000000
		);
	});
}

#[test]
fn deposit_asset_should_work_2() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(FarmExtend::create_pool(
			Origin::signed(ALICE),
			DOT,
			100,
			1100,
			1_000_000_000,
			DICO
		));

		System::set_block_number(120);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		let currency_amount: Balance = (1100 - 100) * 1_000_000_000;
		let mut pool_extend_info =
			PoolExtendInfo::new(DOT, currency_amount, ALICE, 100, 1100, 1_000_000_000, 120, DICO);
		pool_extend_info.total_stake_amount = 1000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 2000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		System::set_block_number(150);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 0,));
		// acc_reward_per_share = (150 - 120) * 1000000000 * 1e12 / 2000000000000 = 15000000000
		pool_extend_info.acc_reward_per_share = 15_000_000_000;
		pool_extend_info.last_reward_block = 150;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		// reward_debt = 15000000000 * 1000_000_000_000 / 1e12 = 15000000000
		assert_eq!(Currency::free_balance(DOT, &BOB), DEFAULT_ASSET_AMOUNT + 15000000000);
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 15000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 0,));
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		// reward_debt = 15000000000 * 1000_000_000_000 / 1e12 = 15000000000
		assert_eq!(Currency::free_balance(DOT, &ALICE), DEFAULT_ASSET_AMOUNT + 15000000000 - 1_000_000_000 * 1000);
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 15000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));
	});
}

#[test]
fn withdraw_asset_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(FarmExtend::create_pool(
			Origin::signed(ALICE),
			DOT,
			100,
			1100,
			1_000_000_000,
			DICO
		));

		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		let currency_amount: Balance = (1100 - 100) * 1_000_000_000;
		let mut pool_extend_info =
			PoolExtendInfo::new(DOT, currency_amount, ALICE, 100, 1100, 1_000_000_000, 100, DICO);
		pool_extend_info.total_stake_amount = 1000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 1000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &ALICE),
			DEFAULT_ASSET_AMOUNT - 1000_000_000_000
		);
		expect_events(vec![Event::AssetDeposited(ALICE, 0, 1000_000_000_000).into()]);

		System::set_block_number(100);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 2000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 2000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &BOB),
			DEFAULT_ASSET_AMOUNT - 1000_000_000_000
		);

		System::set_block_number(200);
		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(BOB), 0, 0,));
		pool_extend_info.total_stake_amount = 2000_000_000_000;
		pool_extend_info.last_reward_block = 200;
		// acc_reward_per_share = (200 - 100) * 1000000000 * 1e12 / 2000000000000 = 50000000000
		pool_extend_info.acc_reward_per_share = 50000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));
		// reward_debt = 1000000000000 * 50000000000 / 1e12 = 50000000000
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 50000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DICO, &module_id_account), 2000_000_000_000);
		assert_eq!(
			Currency::free_balance(DICO, &BOB),
			DEFAULT_ASSET_AMOUNT - 1000_000_000_000
		);

		// pending_reward = 1000000000000 * 50000000000 / 1e12 - 0 = 50000000000
		assert_eq!(
			Currency::free_balance(DOT, &module_id_account),
			currency_amount - 50000000000
		);
		assert_eq!(Currency::free_balance(DOT, &BOB), DEFAULT_ASSET_AMOUNT + 50000000000);

		System::set_block_number(2000);
		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 1000_000_000_000;
		pool_extend_info.last_reward_block = 1100;
		// add_reward_per_share = (1100 - 200) * 1000000000 * 1e12 / 2000000000000 = 450000000000
		// acc_reward_per_share = 50000000000 + 450000000000 = 500000000000
		pool_extend_info.acc_reward_per_share = 500000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		// reward_debt = 0 * 500000000000 / 1e12 = 0
		let participant_extend = ParticipantExtend::new(0, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));

		// pending_reward = 1000000000000 * 500000000000 / 1e12 - 0 = 500000000000
		assert_eq!(
			Currency::free_balance(DOT, &module_id_account),
			currency_amount - 50000000000 - 500000000000
		);
		assert_eq!(
			Currency::free_balance(DOT, &ALICE),
			DEFAULT_ASSET_AMOUNT - currency_amount + 500000000000
		);

		System::set_block_number(3000);
		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(BOB), 0, 500_000_000_000,));
		pool_extend_info.total_stake_amount = 500_000_000_000;
		pool_extend_info.last_reward_block = 1100;
		// add_reward_per_share = (1100 - 1100) * 1000000000 * 1e12 / 3000000000000 = 0
		// acc_reward_per_share = 0 + 500000000000 = 500000000000
		pool_extend_info.acc_reward_per_share = 500000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		// reward_debt = 500000000000 * 500000000000 / 1e12 = 250000000000
		let participant_extend = ParticipantExtend::new(500_000_000_000, 250000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		// pending_reward = 1000000000000 * 500000000000 / 1e12 - 50000000000 = 450000000000
		assert_eq!(Currency::free_balance(DOT, &module_id_account), 0);
		assert_eq!(
			Currency::free_balance(DOT, &BOB),
			DEFAULT_ASSET_AMOUNT + 50000000000 + 450000000000
		);

		System::set_block_number(4000);
		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(BOB), 0, 500_000_000_000,));
		pool_extend_info.total_stake_amount = 0;
		pool_extend_info.last_reward_block = 1100;
		// add_reward_per_share = (1100 - 1100) * 1000000000 * 1e12 / 3000000000000 = 0
		// acc_reward_per_share = 0 + 500000000000 = 500000000000
		pool_extend_info.acc_reward_per_share = 500000000000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		// reward_debt = 0 * 500000000000 / 1e12 = 0
		let participant_extend = ParticipantExtend::new(0, 0);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		// pending_reward = 500000000000 * 500000000000 / 1e12 - 250000000000 = 0
		assert_eq!(Currency::free_balance(DOT, &module_id_account), 0);
		assert_eq!(
			Currency::free_balance(DOT, &BOB),
			DEFAULT_ASSET_AMOUNT + 50000000000 + 450000000000
		);
	});
}


#[test]
fn withdraw_asset_should_work_2() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(FarmExtend::create_pool(
			Origin::signed(ALICE),
			DOT,
			100,
			1100,
			1_000_000_000,
			DICO
		));

		System::set_block_number(120);
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(ALICE), 0, 1000_000_000_000,));
		let currency_amount: Balance = (1100 - 100) * 1_000_000_000;
		let mut pool_extend_info =
			PoolExtendInfo::new(DOT, currency_amount, ALICE, 100, 1100, 1_000_000_000, 120, DICO);
		pool_extend_info.total_stake_amount = 1000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		assert_ok!(FarmExtend::deposit_asset(Origin::signed(BOB), 0, 1000_000_000_000,));
		pool_extend_info.total_stake_amount = 2000_000_000_000;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));

		System::set_block_number(150);
		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(BOB), 0, 0,));
		// acc_reward_per_share = (150 - 120) * 1000000000 * 1e12 / 2000000000000 = 15000000000
		pool_extend_info.acc_reward_per_share = 15_000_000_000;
		pool_extend_info.last_reward_block = 150;
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		// reward_debt = 15000000000 * 1000_000_000_000 / 1e12 = 15000000000
		assert_eq!(Currency::free_balance(DOT, &BOB), DEFAULT_ASSET_AMOUNT + 15000000000);
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 15000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, BOB), Some(participant_extend));

		assert_ok!(FarmExtend::withdraw_asset(Origin::signed(ALICE), 0, 0,));
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		// reward_debt = 15000000000 * 1000_000_000_000 / 1e12 = 15000000000
		assert_eq!(Currency::free_balance(DOT, &ALICE), DEFAULT_ASSET_AMOUNT + 15000000000 - 1_000_000_000 * 1000);
		let participant_extend = ParticipantExtend::new(1000_000_000_000, 15000000000);
		assert_eq!(ParticipantExtends::<Test>::get(0, ALICE), Some(participant_extend));
	});
}

