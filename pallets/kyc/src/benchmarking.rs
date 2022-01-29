//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

#[allow(unused)]
use crate::Pallet as KYC;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const CURVE_PUBLIC_KEY: [u8; 32] = [
	2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn add_adminer<T: Config>(r: u32) -> Result<(), &'static str> {
	for i in 0..r {
		let adminer: T::AccountId = account("adminer", i, SEED);
		// let _ = T::Currency::make_free_balance_be(&adminer, BalanceOf::<T>::max_value());

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

benchmarks! {
	add_ias {
		let r in 1 .. T::MaxIAS::get() - 1 => add_adminer::<T>(r)?;

		let adminer: T::AccountId = account("adminer", r, SEED);
		let adminer_ias_info = IASInfo {
			account: adminer.clone(),
			fee: 100u32.into(),
			curve_public_key: CURVE_PUBLIC_KEY,
			fields: KYCFields::Area,
		};

		ensure!(IASListOf::<T>::get(KYCFields::Area).len() as u32 == r, "IAS not set up correctly.");
	}: _(RawOrigin::Signed(adminer), r, adminer_ias_info)
	verify {
		ensure!(IASListOf::<T>::get(KYCFields::Area).len() as u32 == r + 1, "IAS not added.");
	}

}

impl_benchmark_test_suite!(KYC, crate::mock::new_test_ext(), crate::mock::Test,);
