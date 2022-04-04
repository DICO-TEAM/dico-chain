#![cfg(test)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, debug};

fn initialize() {
	Balances::set_balance(Origin::root(), Alice, 100_000 * DOLLARS, 100 * DOLLARS).unwrap();
	Balances::set_balance(Origin::root(), Bob, 100_000 * DOLLARS, 100 * DOLLARS).unwrap();
	IcoMaxUsdtAmount::<Test>::put(1000 * DOLLARS);
	Currencies::create_asset(
		Origin::signed(Alice),
		KSM,
		100_000 * DOLLARS,
		Some(DicoAssetMetadata {
			name: b"KUSAMA".to_vec(),
			symbol: b"KSM".to_vec(),
			decimals: 12,
		}),
	)
	.unwrap();

	Currencies::create_asset(
		Origin::signed(Bob),
		NEW_USDT,
		100_000 * DOLLARS,
		Some(DicoAssetMetadata {
			name: b"NEW_USDT".to_vec(),
			symbol: b"NEW_USDT".to_vec(),
			decimals: 8,
		}),
	)
	.unwrap();

	Currencies::create_asset(
		Origin::signed(Bob),
		DOT,
		100_000 * DOLLARS,
		Some(DicoAssetMetadata {
			name: b"POLKADOT".to_vec(),
			symbol: b"DOT".to_vec(),
			decimals: 12,
		}),
	)
	.unwrap();
	Currencies::transfer(Origin::signed(Bob), Alice, DOT, 5000 * DOLLARS).unwrap();

	Ico::<Test>::insert(
		KSM,
		1,
		IcoInfo {
			desc: vec![],
			start_time: Some(0u64),
			is_already_kyc: false,
			initiator: Alice,
			total_usdt: 3500_0000 * USD,
			tag: None,
			is_terminated: false,
			project_name: b"KUSAMA".to_vec(),
			token_symbol: b"KSM".to_vec(),
			decimals: 12,
			index: Some(1u32),
			already_released_proportion: Default::default(),
			currency_id: KSM,
			official_website: vec![],
			user_ico_max_times: 2,
			is_must_kyc: false,
			total_issuance: 10000 * DOLLARS,
			total_circulation: 1000 * DOLLARS,
			ico_duration: NewDAYS,
			total_ico_amount: 1000 * DOLLARS,
			user_min_amount: 100 * DOLLARS,
			user_max_amount: 500 * DOLLARS,
			exchange_token: NEW_USDT,
			exchange_token_total_amount: 10000 * DOLLARS,
			exclude_area: vec![],
			lock_proportion: Default::default(),
			unlock_duration: NewDAYS,
			per_duration_unlock_amount: 0 * DOLLARS,
		},
	);
}

#[test]
fn initiate_ico_should_work() {
	new_test_ext().execute_with(|| {
		let info = IcoParameters {
			desc: vec![],
			currency_id: DOT,
			official_website: vec![],
			is_must_kyc: false,
			user_ico_max_times: 2,
			total_issuance: 10000 * DOLLARS,
			total_circulation: 1000 * DOLLARS,
			ico_duration: NewDAYS,
			total_ico_amount: 1000 * DOLLARS,
			user_min_amount: 100 * DOLLARS,
			user_max_amount: 500 * DOLLARS,
			exchange_token: KSM,
			exchange_token_total_amount: 10000 * DOLLARS,
			exclude_area: vec![],
			lock_proportion: Default::default(),
			unlock_duration: NewDAYS,
			per_duration_unlock_amount: 0 * DOLLARS,
		};
		initialize();
		assert_ok!(IcoTest::initiate_ico(Origin::signed(Alice), info));
	});
}

#[test]
fn join_should_work() {
	new_test_ext().execute_with(|| {
		Ico::<Test>::insert(
			KSM,
			1,
			IcoInfo {
				desc: vec![],
				start_time: Some(0u64),
				is_already_kyc: false,
				initiator: Alice,
				total_usdt: 1000,
				tag: None,
				is_terminated: false,
				project_name: b"KUSAMA".to_vec(),
				token_symbol: b"KSM".to_vec(),
				decimals: 12,
				index: Some(1u32),
				already_released_proportion: Default::default(),
				currency_id: KSM,
				official_website: vec![],
				user_ico_max_times: 2,
				is_must_kyc: false,
				total_issuance: 10000 * DOLLARS,
				total_circulation: 1000 * DOLLARS,
				ico_duration: NewDAYS,
				total_ico_amount: 1000 * DOLLARS,
				user_min_amount: 100 * DOLLARS,
				user_max_amount: 500 * DOLLARS,
				exchange_token: NEW_USDT,
				exchange_token_total_amount: 10000 * DOLLARS,
				exclude_area: vec![],
				lock_proportion: Default::default(),
				unlock_duration: NewDAYS,
				per_duration_unlock_amount: 0 * DOLLARS,
			},
		);
		initialize();
		let result = IcoTest::join(Origin::signed(Bob), KSM, 1u32, 100 * DOLLARS, None);
		println!("result: {:?}", result);
	});
}

#[test]
pub fn calculate_total_reward_should_work() {
	new_test_ext().execute_with(|| {
		initialize();
		TotalUsdt::<Test>::put(1_8000_0000 * USD);
		let ico = Ico::<Test>::get(KSM, 1).unwrap();
		print!("ico: {:#?}\n", ico);
		let reward = IcoTest::calculate_total_reward(&ico);
		println!("reward: {:?}", reward);
		assert_eq!(reward, 13750000 * USD + 1);
	});
}

fn split(mut this_time_amount: u64, total_amount: u64, unit: u64) -> Vec<u64> {
	let mut remain = total_amount.clone() % unit.clone();
	let mut num = total_amount / unit.clone();
	println!("num {:?}", num);
	let mut result = vec![];
	loop {
		if this_time_amount.saturating_sub(remain) > 0 {
			result.push(remain);
			this_time_amount -= remain;
			remain = unit.clone();
		} else {
			result.push(this_time_amount);
			break;
		}
	}
	result
}
#[test]
fn main_test() {
	let a = split(45u64, 45u64, 20);
	println!("{:?}", a);
}
