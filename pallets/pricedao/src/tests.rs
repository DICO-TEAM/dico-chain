#![cfg(test)]

use crate::Balance;
use crate::{mock::*, Error};
use frame_support::pallet_prelude::DispatchError::BadOrigin;
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok, log};

#[test]
fn insert_feed_account_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PriceDao::insert_feed_account(Origin::signed(GAVIN), vec![ALICE, BOB]),
			BadOrigin
		);

		assert_ok!(PriceDao::insert_feed_account(
			Origin::signed(ALICE),
			vec![ALICE, BOB, DAVE]
		));
		assert_eq!(
			crate::DepositBalance::<Test>::get(&BOB),
			Some(crate::DepositBalanceInfo {
				amount: 50_000_000_000_000_000,
				expiration: 0
			})
		);

		let set = pallet_oracle::Members::<Test, _>::get();
		println!("----test-----");
		assert_eq!(set.0.len(), 2);
		assert_eq!(set.contains(&ALICE), true);
		assert_eq!(set.contains(&BOB), true);
		assert_eq!(
			<Balances as Currency<AccountId>>::total_balance(&BOB),
			2000_000_000_000_000_000
		);

		assert_eq!(Balances::free_balance(&DAVE), 40_000_000_000_000_000);

		assert_eq!(
			Balances::free_balance(&BOB),
			2000_000_000_000_000_000 - 50_000_000_000_000_000
		);
		assert_eq!(Balances::reserved_balance(&BOB), 50_000_000_000_000_000);
	});
}

#[test]
fn del_feed_account_should_work() {
	// slash
	new_test_ext().execute_with(|| {
		assert_eq!(<Balances as Currency<AccountId>>::total_balance(&BOB), 2000_000_000_000_000_000);
		assert_ok!(PriceDao::insert_feed_account(Origin::signed(ALICE),vec![ALICE,BOB]));

		assert_ok!(PriceDao::del_feed_account(Origin::signed(ALICE),vec![ALICE,BOB]));
		assert_eq!(Balances::total_balance(&BOB), 2000_000_000_000_000_000-50_000_000_000_000_000);
		let set = pallet_oracle::Members::<Test, _>::get();
		assert_eq!(set.0.len(), 0);
		assert_eq!(set.contains(&BOB), false);
		assert_ok!(
			PriceDao::del_feed_account(Origin::signed(ALICE),vec![ALICE])    // invalid
		);
		assert_eq!(Balances::total_balance(&ALICE), 2000_000_000_000_000_000-50_000_000_000_000_000);
		assert_eq!(Balances::total_balance(&ALICE), 2000_000_000_000_000_000-50_000_000_000_000_000);

		assert_eq!(Balances::total_balance(&PriceDao::account_id()), 100_000_000_000_000_000);// Treasury
	});
}

#[test]
fn exit_feed_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(PriceDao::insert_feed_account(Origin::signed(ALICE), vec![ALICE, BOB]));

		assert_noop!(PriceDao::exit_feed(Origin::signed(GAVIN)), Error::<Test>::NoneValue);

		assert_ok!(PriceDao::exit_feed(Origin::signed(BOB)));
		let set = pallet_oracle::Members::<Test, _>::get();
		assert_eq!(set.0.len(), 1);
		assert_eq!(set.contains(&BOB), false);
		assert_eq!(set.contains(&ALICE), true);
		assert_eq!(Balances::total_balance(&BOB), 2000_000_000_000_000_000);
		assert_eq!(Balances::reserved_balance(&BOB), 50_000_000_000_000_000);
		assert_eq!(
			crate::DepositBalance::<Test>::get(&BOB),
			Some(crate::DepositBalanceInfo {
				amount: 50_000_000_000_000_000,
				expiration: 11
			})
		);

		System::set_block_number(11);
		assert_noop!(
			PriceDao::exit_feed(Origin::signed(BOB)),
			Error::<Test>::ExpirationNotEmpty
		);
		assert_eq!(Balances::reserved_balance(&BOB), 50_000_000_000_000_000);
	});
}

#[test]
fn withdraw_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(PriceDao::insert_feed_account(Origin::signed(ALICE), vec![ALICE, BOB]));
		assert_noop!(PriceDao::withdraw(Origin::signed(BOB)), Error::<Test>::ExpirationEmpty);

		assert_ok!(PriceDao::exit_feed(Origin::signed(BOB)));
		System::set_block_number(10);
		assert_noop!(PriceDao::withdraw(Origin::signed(BOB)), Error::<Test>::NotExpired);

		System::set_block_number(11);
		assert_ok!(PriceDao::withdraw(Origin::signed(BOB)));
		assert_eq!(crate::DepositBalance::<Test>::get(&BOB), None);

		assert_noop!(PriceDao::withdraw(Origin::signed(BOB)), Error::<Test>::NoneValue);
		assert_eq!(Balances::reserved_balance(&BOB), 0);
		assert_eq!(Balances::free_balance(&BOB), 2000_000_000_000_000_000);
	});
}

#[test]
fn unlock_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(PriceDao::unlock_price(Origin::signed(ALICE), 1));
	});
}
