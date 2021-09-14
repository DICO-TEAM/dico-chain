#![cfg(test)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, debug};

#[test]
fn test1() {
	new_test_ext().execute_with(|| {
		assert_eq!(IcoTest::now(), 1);
		// debug(&IcoTest::now);
		println!("{:}", IcoTest::now());
		let a = IcoTest::classify_user_amount(100, vec![(55, 100), (37, 70)]);
		// println!("{:?}", a);
		// println!("{:?}", TotalNum::get());
		// TotalNum::put(1000000);
		// println!("{:?}", TotalNum::get());
		assert_eq!(IcoTest::caculate_user_reward(a, 1000, 10000), 1200);

		assert_ok!(IcoTest::join(Origin::signed(1), 1u32, 1u32, 10000u64));

		// assert_ok!(())
	});
}

#[test]
fn unrelease_test() {
	new_test_ext().execute_with(|| {
		println!("first: {:#?}", RequestReleaseInfo::<Test>::get());
		IcoTest::request_release(Origin::signed(1), 1, 2, Percent::from_percent(5u8));
		println!("{:#?}", RequestReleaseInfo::<Test>::get());
	});
}

#[test]
fn calculate_total_power_test() {
	new_test_ext().execute_with(|| {
		let ico = IcoInfo {
			start_time: None,
			is_identity: false,
			initiator: 1,
			total_usdt: 100_0000,
			is_terminated: false,
			project_name: vec![],
			token_symbol: vec![],
			decimals: 0,
			index: Some(1),
			already_released_proportion: Default::default(),
			currency_id: 1,
			logo_url: vec![],
			official_website: vec![],
			user_ico_max_count: 0,
			is_must_kyc: false,
			total_issuance: 100_0000,
			total_circulation: 100_0000,
			ico_duration: 100,
			total_ico_amount: 100_0000,
			user_min_amount: 10000,
			user_max_amount: 50000,
			exchange_token: 2,
			total_exchange_amount: 10_0000,
			exclude_nation: vec![Countries::Chain],
			lock_proportion: Default::default(),
			unlock_duration: 0,
			per_duration_unlock_amount: 0,
		};

		TotalUsdt::<Test>::put(250_0000);

		UnReleaseAssets::<Test>::insert(
			ico.initiator,
			vec![UnRelease {
				currency_id: ico.currency_id,
				index: ico.index.unwrap(),
				unreleased_currency_id: ico.exchange_token,
				tags: vec![(5000, 100_000)],
				total: 100000,
				released: 8000,
				is_already_get_reward: None,
			}],
		);

		let initiator_info = IcoTest::get_unrelease_assets(ico.initiator, ico.currency_id, ico.index.unwrap());
		// println!("initiator_info: {:?}", initiator_info);
		assert_eq!(
			initiator_info.is_some() && initiator_info.unwrap().total == 10_0000,
			true
		);

		let result = IcoTest::calculate_total_power(ico);

		println!("result: {:}", result);
		println!("total_usdt: {:}", TotalUsdt::<Test>::get());
	});
}
