#![allow(deprecated)]

use super::*;
use crate::{Config, Weight};
use frame_support::{log, traits::Get};

pub mod v2 {
	use super::*;

	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		log::info!("👜 KYC module  pre_migrate ✅",);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
		log::info!("👜 KYC module  post_migrate ✅",);
		Ok(())
	}

	pub fn migrate<T: Config>() -> Weight {
		log::info!("👜 KYC module  start migrate ✅",);

		let service_deposit = T::ServiceDeposit::get();

		KYCOf::<T>::iter().for_each(|(account, registration)| {
			T::Currency::unreserve(&account, registration.deposit);
		});

		IASListOf::<T>::iter().for_each(|(_key, ias_list)| {
			for ias in ias_list.iter() {
				if ias.is_some() {
					let ias_clone = ias.clone();
					let ias_account = &ias_clone.unwrap().account;
					T::Currency::unreserve(&ias_account, service_deposit);
				}
			}
		});

		SwordHolderOf::<T>::iter().for_each(|(_key, ias_list)| {
			for ias in ias_list.iter() {
				if ias.is_some() {
					let ias_clone = ias.clone();
					let ias_account = &ias_clone.unwrap().account;
					T::Currency::unreserve(&ias_account, service_deposit);
				}
			}
		});

		AreaData::<T>::remove_all(Some(100));
		KYCOf::<T>::remove_all(Some(100));
		BlackListOf::<T>::remove_all(Some(100));
		IASListOf::<T>::remove_all(Some(100));
		SwordHolderOf::<T>::remove_all(Some(100));
		RecordsOf::<T>::remove_all(Some(100));
		UniqueIdOf::<T>::remove_all(Some(100));
		MessageList::<T>::remove_all(Some(100));
		ApplicationFormList::<T>::remove_all(Some(100));
		T::DbWeight::get().reads_writes(1, 1)
	}
}
