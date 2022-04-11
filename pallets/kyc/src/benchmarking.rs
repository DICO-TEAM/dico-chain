//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::types::*;
#[allow(unused)]
use crate::Pallet as KYC;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use scale_info::prelude::string::String;
use sp_runtime::traits::Bounded;

const SEED: u32 = 0;
const CURVE_PUBLIC_KEY: [u8; 32] = [
	2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn alice_message() -> [u8; 128] {
	[
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
		1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
	]
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn add_adminer<T: Config>(r: u32) -> Result<(), &'static str> {
	for i in 0..r {
		let adminer: T::AccountId = account("adminer", i, SEED);
		// let _ = T::Currency::make_free_balance_be(&adminer, BalanceOf::<T>::max_value());
		let _ = T::Currency::make_free_balance_be(&adminer, BalanceOf::<T>::max_value());
		let adminer_ias_info = IASInfo {
			account: adminer.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		KYC::<T>::add_ias(RawOrigin::Root.into(), i.into(), adminer_ias_info.clone())?;
		KYC::<T>::add_sword_holder(RawOrigin::Root.into(), i.into(), adminer_ias_info.clone())?;
	}

	assert_eq!(IASListOf::<T>::get(KYCFields::Area).len(), r as usize);
	assert_eq!(SwordHolderOf::<T>::get(KYCFields::Area).len(), r as usize);
	Ok(())
}

fn create_kyc() -> KYCInfo {
	let kyc = KYCInfo {
		name: Vec::from(String::from("alice")),
		area: AreaCode::AF,
		curve_public_key: Vec::from(String::from("alice")),
		email: vec![],
	};
	return kyc;
}

benchmarks! {
	set_kyc {
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;
		let caller: T::AccountId = account("caller", r, SEED);
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
	}: _(RawOrigin::Signed(caller.clone()), create_kyc())
	verify {
		assert_last_event::<T>(Event::<T>::KYCSet(caller.clone()).into());
	}

	clear_kyc {
		let caller: T::AccountId = whitelisted_caller();
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		//TODO: add judgement
		let caller_kyc = create_kyc();
		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		ensure!(!KYCOf::<T>::contains_key(&caller), "KYC not cleared.");
	}

	remove_kyc {
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;
		let caller: T::AccountId = account("caller", r, SEED);
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
	}: _(RawOrigin::Root,caller.clone(),Black::Unknown)
	verify {
		assert_last_event::<T>(Event::<T>::KYCRemove(caller.clone()).into());
	}

	apply_certification {
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		let caller: T::AccountId = whitelisted_caller();
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let caller_kyc = create_kyc();
		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
	}: _(RawOrigin::Signed(caller.clone()),KYCFields::Area, 500u32.into())
		verify {
			assert_last_event::<T>(Event::<T>::ApplyCertification(caller.clone()).into());
	}



	add_ias {
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		let adminer: T::AccountId = account("add_ias", r, SEED);
		let adminer_ias_info = IASInfo {
			account: adminer.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&adminer, BalanceOf::<T>::max_value());
		ensure!(IASListOf::<T>::get(KYCFields::Area).len() as u32 == r, "IAS not set up correctly.");
	}: _(RawOrigin::Root, r, adminer_ias_info)
	verify {
		ensure!(IASListOf::<T>::get(KYCFields::Area).len() as u32 == r + 1, "IAS not added.");
	}



	add_sword_holder {
		let r in 1 .. T::MaxSwordHolder::get() - 1 => add_adminer::<T>(r)?;

		let adminer: T::AccountId = account("add_sword_holder", r, SEED);
		let adminer_ias_info = IASInfo {
			account: adminer.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&adminer, BalanceOf::<T>::max_value());
		ensure!(SwordHolderOf::<T>::get(KYCFields::Area).len() as u32 == r, "sword holder  not set up correctly.");
	}: _(RawOrigin::Root, r, adminer_ias_info)
	verify {
		ensure!(SwordHolderOf::<T>::get(KYCFields::Area).len() as u32 == r + 1, "sword holder not added.");
	}

		ias_set_fee {
		let caller: T::AccountId = whitelisted_caller();
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		let adminer_ias_info = IASInfo {
			account: caller.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		KYC::<T>::add_ias(RawOrigin::Root.into(),r, adminer_ias_info)?;

		let ias_list = IASListOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().fee == 100u32.into(), "Fee already set.");
	}: _(RawOrigin::Signed(caller), KYCFields::Area, 1000u32.into())
	verify {
		let ias_list = IASListOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().fee == 1000u32.into(), "Fee not changed.");
	}

	sword_holder_set_fee {
		let caller: T::AccountId = whitelisted_caller();
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		let adminer_ias_info = IASInfo {
			account: caller.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		KYC::<T>::add_sword_holder(RawOrigin::Root.into(),r, adminer_ias_info)?;

		let ias_list = SwordHolderOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().fee == 100u32.into(), "Fee already set.");
	}: _(RawOrigin::Signed(caller), KYCFields::Area, 1000u32.into())
	verify {
		let ias_list = SwordHolderOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().fee == 1000u32.into(), "Fee not changed.");
	}

	kill_ias {

		let r in 0 .. T::MaxIAS::get()-1 => add_adminer::<T>(r)?;

		let caller: T::AccountId = whitelisted_caller();

		let adminer_ias_info = IASInfo {
			account: caller.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::add_ias(RawOrigin::Root.into(),r, adminer_ias_info)?;


		let ias_list = IASListOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().account == caller.clone(), "IAS not add.");

	}: _(RawOrigin::Root, r, KYCFields::Area)
	verify {
		let ias_list = IASListOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize ].as_ref().unwrap().account != caller.clone(), "IAS not removed.");
	}

	kill_sword_holder {

		let r in 0 .. T::MaxSwordHolder::get()-1 => add_adminer::<T>(r)?;

		let caller: T::AccountId = whitelisted_caller();

		let adminer_ias_info = IASInfo {
			account: caller.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::add_sword_holder(RawOrigin::Root.into(),r, adminer_ias_info)?;


		let ias_list = SwordHolderOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize].as_ref().unwrap().account == caller.clone(), "SwordHolder not add.");

	}: _(RawOrigin::Root, r, KYCFields::Area)
	verify {
		let ias_list = SwordHolderOf::<T>::get(KYCFields::Area);
		ensure!(ias_list[r as usize ].as_ref().unwrap().account != caller.clone(), "SwordHolder not removed.");
	}

	request_judgement {
		let n in 1 .. T::MaxIAS::get();

		let adminer1: T::AccountId = account("target", 0, SEED);
		let adminer2: T::AccountId = account("target", 1, SEED);

		let adminer1_ias_info = IASInfo {
			account: adminer1.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		let adminer2_ias_info = IASInfo {
			account: adminer2.clone(),
			fee: 200u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};


		let _ = T::Currency::make_free_balance_be(&adminer1, BalanceOf::<T>::max_value());
		let _ = T::Currency::make_free_balance_be(&adminer2, BalanceOf::<T>::max_value());

		KYC::<T>::add_ias(RawOrigin::Root.into(), 0u32.into(), adminer1_ias_info.clone())?;
		KYC::<T>::add_sword_holder(RawOrigin::Root.into(), 0u32.into(), adminer2_ias_info.clone())?;

		let caller: T::AccountId = account("target", 2, SEED);
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let caller_kyc = create_kyc();

		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
		KYC::<T>::apply_certification(caller_origin.clone(), KYCFields::Area, 500u32.into())?;
	}: _(RawOrigin::Signed(caller.clone()),KYCFields::Area,0u32, alice_message())
	verify {
		assert_last_event::<T>(Event::<T>::JudgementRequested(caller.clone(), 0u32).into());
	}


	ias_provide_judgement {
		let n in 1 .. T::MaxIAS::get();

		let adminer1: T::AccountId = account("target", 0, SEED);
		let adminer2: T::AccountId = account("target", 1, SEED);

		let adminer1_ias_info = IASInfo {
			account: adminer1.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		let adminer2_ias_info = IASInfo {
			account: adminer2.clone(),
			fee: 200u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		let _ = T::Currency::make_free_balance_be(&adminer1, BalanceOf::<T>::max_value());
		let _ = T::Currency::make_free_balance_be(&adminer2, BalanceOf::<T>::max_value());

		KYC::<T>::add_ias(RawOrigin::Root.into(), 0u32.into(), adminer1_ias_info.clone())?;
		KYC::<T>::add_sword_holder(RawOrigin::Root.into(), 0u32.into(), adminer2_ias_info.clone())?;

		let caller: T::AccountId = account("target", 2, SEED);
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let caller_kyc = create_kyc();

		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
		KYC::<T>::apply_certification(caller_origin.clone(), KYCFields::Area, 500u32.into())?;
		KYC::<T>::request_judgement(caller_origin.clone(),KYCFields::Area,0u32, alice_message())?;
	}: _(RawOrigin::Signed(adminer1.clone()),KYCFields::Area,0u32,caller.clone(),Judgement::PASS,"1".into(), alice_message())
	verify {
			assert_last_event::<T>(Event::<T>::JudgementGiven(caller, 0u32).into());
	}



	sword_holder_provide_judgement {
		let n in 1 .. T::MaxIAS::get();

		let adminer1: T::AccountId = account("target", 0, SEED);
		let adminer1_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(adminer1.clone()));
		let adminer2: T::AccountId = account("target", 1, SEED);

		let adminer1_ias_info = IASInfo {
			account: adminer1.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		let adminer2_ias_info = IASInfo {
			account: adminer2.clone(),
			fee: 200u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		let _ = T::Currency::make_free_balance_be(&adminer1, BalanceOf::<T>::max_value());
		let _ = T::Currency::make_free_balance_be(&adminer2, BalanceOf::<T>::max_value());

		KYC::<T>::add_ias(RawOrigin::Root.into(), 0u32.into(), adminer1_ias_info.clone())?;
		KYC::<T>::add_sword_holder(RawOrigin::Root.into(), 0u32.into(), adminer2_ias_info.clone())?;

		let caller: T::AccountId = account("target", 2, SEED);
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		let caller_kyc = create_kyc();

		let _ = T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());

		KYC::<T>::set_kyc(caller_origin.clone(), create_kyc())?;

		ensure!(KYCOf::<T>::contains_key(&caller), "KYC does not exist.");
		KYC::<T>::apply_certification(caller_origin.clone(), KYCFields::Area, 500u32.into())?;
		KYC::<T>::request_judgement(caller_origin.clone(),KYCFields::Area,0u32, alice_message())?;
		KYC::<T>::ias_provide_judgement(adminer1_origin.clone(),KYCFields::Area,0u32,caller.clone(),Judgement::PASS,"1".into(), alice_message())?;
	}: _(RawOrigin::Signed(adminer2.clone()),KYCFields::Area,0u32,caller.clone(),Authentication::Success,"1".into())
	verify {
			assert_last_event::<T>(Event::<T>::AuthenticationGiven(caller, 0u32).into());
	}

}

impl_benchmark_test_suite!(KYC, crate::mock::new_test_ext(), crate::mock::Test,);
