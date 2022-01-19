#![cfg(test)]
use super::*;
use crate::mock::{Origin, Call, *};
use frame_support::{assert_noop, assert_ok, debug};
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;

fn proposal() -> H256 {
	let proposal = Call::IcoTest(ico::Call::terminate_ico {
		currency_id: KSM,
		index: 1u32,
	});
	let hash =BlakeTwo256::hash_of(&proposal);
	Dao::propose(Origin::signed(Alice), KSM, 1, Percent::from_percent(50u8), Box::new(proposal), vec![], 15000u32);
	assert_eq!(ProposalOf::<Test>::contains_key(KSM, hash), true);
	hash
}

#[test]
fn propose_should_work() {
	new_test_ext().execute_with(|| {
		let _ = proposal();

	});
}

#[test]
fn vote_should_work() {
	new_test_ext().execute_with(|| {
		let hash = proposal();
		Dao::vote(Origin::signed(Bob), KSM, 1u32, hash, 0u32, true);
		Dao::vote(Origin::signed(Herry), KSM, 1u32, hash, 0u32, true);
		assert_ok!(Dao::close(Origin::signed(Bob), KSM, 1u32, hash, 0u32, 100000u64, 10000u32));
		assert_eq!(Voting::<Test>::contains_key(KSM, hash), false);
	});

}


