// externing crate for test-only use
#[cfg(test)]
use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok};

fn alice_kyc() -> KYCInfo {
	KYCInfo {
		name: Vec::from(String::from("alice")),
		area: AreaCode::AF,
		curve_public_key: Vec::from(String::from("alice")),
		email: vec![],
	}
}

fn bob_kyc() -> KYCInfo {
	KYCInfo {
		name: Vec::from(String::from("bob")),
		area: AreaCode::AF,
		curve_public_key: Vec::from(String::from("bob")),
		email: vec![],
	}
}

fn alice_ias_info() -> IASInfo<AccountId, Balance> {
	IASInfo {
		account: 1,
		fee: 100,
		curve_public_key: [
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		],
		fields: KYCFields::Name,
	}
}

fn bob_ias_info() -> IASInfo<AccountId, Balance> {
	IASInfo {
		account: 1,
		fee: 100,
		curve_public_key: [
			2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		],
		fields: KYCFields::Name,
	}
}

fn alice_message() -> [u8; 128] {
	[
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
	]
}

fn bob_message() -> [u8; 128] {
	[
		2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
	]
}

#[test]
fn initial_state() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(&ALICE), 100);
		assert_eq!(Balances::free_balance(&BOB), 100);
		assert_eq!(Balances::free_balance(&CHARLIE), 100);
		assert_eq!(Balances::free_balance(&DAVE), 100);
		assert_eq!(Balances::free_balance(&EVE), 100);
	});
}

#[test]
fn add_ias_should_work() {
	use sp_runtime::DispatchError;
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(
			KYC::add_ias(Origin::signed(1), 0, alice_ias_info()),
			DispatchError::BadOrigin
		);
		// out of bound
		assert_noop!(
			KYC::add_ias(Origin::root(), 1, alice_ias_info()),
			Error::<Test>::OutofBounds
		);
	})
}

#[test]
fn add_sword_holder_should_work() {
	use sp_runtime::DispatchError;
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(
			KYC::add_sword_holder(Origin::signed(1), 1, alice_ias_info()),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn kill_ias_should_work() {
	use sp_runtime::DispatchError;
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(
			KYC::kill_ias(Origin::signed(1), 1, KYCFields::Name),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn kill_sword_holder_should_work() {
	use sp_runtime::DispatchError;
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(
			KYC::kill_ias(Origin::signed(1), 1, KYCFields::Name),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn remove_kyc_should_work() {
	use sp_runtime::DispatchError;
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::remove_kyc(Origin::signed(10), 1, Black::Cheat),
			DispatchError::BadOrigin
		);
		assert_noop!(KYC::remove_kyc(Origin::root(), 1, Black::Cheat), Error::<Test>::NoKYC);
		assert_ok!(KYC::set_kyc(Origin::signed(1), alice_kyc()));
		assert_ok!(KYC::remove_kyc(Origin::root(), 1, Black::Cheat));
		assert_noop!(
			KYC::remove_kyc(Origin::root(), 1, Black::Cheat),
			Error::<Test>::Blacklisted
		);
	})
}

#[test]
fn set_kyc_should_work() {
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_ok!(KYC::set_kyc(Origin::signed(1), alice_kyc()));
		assert_noop!(KYC::set_kyc(Origin::signed(1), alice_kyc()), Error::<Test>::KYCFound);
		assert_ok!(KYC::remove_kyc(Origin::root(), 1, Black::Cheat));
		assert_noop!(KYC::set_kyc(Origin::signed(1), alice_kyc()), Error::<Test>::Blacklisted);
	})
}

#[test]
fn clear_kyc_should_work() {
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(KYC::clear_kyc(Origin::signed(1)), Error::<Test>::NotFound);
		assert_ok!(KYC::set_kyc(Origin::signed(1), alice_kyc()));
		assert_ok!(KYC::clear_kyc(Origin::signed(1)));
	})
}

#[test]
fn apply_certification_should_work() {
	new_test_ext().execute_with(|| {
		// sudo or election
		assert_noop!(
			KYC::apply_certification(Origin::signed(1), KYCFields::Name, 100),
			Error::<Test>::NoKYC
		);
		assert_ok!(KYC::set_kyc(Origin::signed(1), alice_kyc()));
		assert_ok!(KYC::remove_kyc(Origin::root(), 1, Black::Cheat));
		assert_noop!(
			KYC::apply_certification(Origin::signed(1), KYCFields::Name, 100),
			Error::<Test>::Blacklisted
		);
	})
}

#[test]
fn request_judgement_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::request_judgement(Origin::signed(1), KYCFields::Name, 0, alice_message()),
			Error::<Test>::NoApplication
		);
	})
}

#[test]
fn ias_set_fee_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::ias_set_fee(Origin::signed(1), KYCFields::Name, 100),
			Error::<Test>::InsufficientPermissions
		);
	})
}

#[test]
fn ias_provide_judgement_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::ias_provide_judgement(
				Origin::signed(1),
				KYCFields::Name,
				1,
				10,
				Judgement::PASS,
				vec![1, 2, 3],
				alice_message()
			),
			Error::<Test>::InsufficientPermissions
		);
	})
}

#[test]
fn sword_holder_set_fee_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::sword_holder_set_fee(Origin::signed(1), KYCFields::Name, 100),
			Error::<Test>::InsufficientPermissions
		);
	})
}

#[test]
fn sword_holder_provide_judgement_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KYC::sword_holder_provide_judgement(
				Origin::signed(1),
				KYCFields::Name,
				1,
				10,
				Authentication::Success,
				vec![1, 2, 3]
			),
			Error::<Test>::InsufficientPermissions
		);
	})
}
