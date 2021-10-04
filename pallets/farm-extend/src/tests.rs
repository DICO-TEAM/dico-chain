//! Unit tests for the farm-extend module.

#![cfg(test)]

use super::*;
pub use crate::mock::{
	Currency, Event as TestEvent, ExtBuilder, Origin,
	System, Test, DOT, ALICE, BOB, USDT, DICO, FarmExtend,
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

		let currency_amount: Balance = (1100 - 100 + 1) * 1_000_000_000;
		let pool_extend_info = PoolExtendInfo::new(
			DOT,
			currency_amount,
			ALICE,
			100,
			1100,
			1_000_000_000,
			100,
			DICO,
			PoolExtendStatus::Pending
		);
		assert_eq!(NextPoolExtendId::<Test>::get(), 1);
		assert_eq!(PoolExtends::<Test>::get(0), Some(pool_extend_info));
		expect_events(vec![Event::PoolExtendCreated(
			ALICE,
			0,
			DOT,
			currency_amount,
			DICO
		).into()]);

		let module_id_account = FarmExtend::account_id();
		assert_eq!(Currency::free_balance(DOT, &module_id_account), currency_amount);
		assert_eq!(Currency::free_balance(DOT, &ALICE), DEFAULT_ASSET_AMOUNT - currency_amount);
	});
}