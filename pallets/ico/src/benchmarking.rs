#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks_instance, benchmarks, impl_benchmark_test_suite, whitelist_account, whitelisted_caller,
benchmarks_instance_pallet};
use frame_support::{assert_ok};

use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use crate::Pallet as Ico;
use currencies::{Pallet as CurrenciesPallet, DicoAssetMetadata};

const SEED: u32 = 0;

fn get_alice<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	T::MultiCurrency::deposit(50, &caller, (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>());
	caller

}

fn get_bob<T: Config>() -> T::AccountId {
	let caller: T::AccountId = account("bob", 1, SEED);
	T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &caller, (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>());
	T::MultiCurrency::deposit(T::UsdtCurrencyId::get(), &caller, (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>());
	caller
}

fn get_haha<T: Config>() -> T::AccountId {
	let caller: T::AccountId = account("haha", 1, SEED);
	T::MultiCurrency::deposit(T::UsdtCurrencyId::get(), &caller, (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>());
	T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &caller, (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>());
	caller
}

fn look_up<T: Config>(who: T::AccountId) -> <T::Lookup as StaticLookup>::Source {
	T::Lookup::unlookup(who)
}

fn set_ico<T: Config>() -> CurrencyId {
	let alice: T::AccountId = get_alice::<T>();
	let currency_id = T::UsdtCurrencyId::get();
	T::CurrenciesHandler::do_create(alice.clone(), currency_id, Some(DicoAssetMetadata {
		name: vec![1; 4],
		symbol: vec![1; 4],
		decimals: 12
	}), (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), false);
	let bob: T::AccountId = get_bob::<T>();
	T::MultiCurrency::deposit(currency_id, &bob, 10000u128.saturated_into::<MultiBalanceOf<T>>());
	let ico_info = IcoParameters {
		desc: vec![],
		currency_id: T::GetNativeCurrencyId::get(),
		official_website: vec![],
		is_must_kyc: false,
		user_ico_max_times: 2,
		total_issuance: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		total_circulation: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		ico_duration: T::BlockNumber::from(100u32),
		total_ico_amount: (1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		user_min_amount: (100 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		user_max_amount: (500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		exchange_token: currency_id,
		exchange_token_total_amount: (1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
		exclude_area: vec![],
		lock_proportion: Default::default(),
		unlock_duration: T::BlockNumber::from(0u32),
		per_duration_unlock_amount: MultiBalanceOf::<T>::from(0u32),
	};
	assert_ok!(Ico::<T>::initiate_ico(RawOrigin::Signed(alice.clone()).into(), ico_info));
	T::GetNativeCurrencyId::get()
}

fn get_ico<T: Config>() -> (CurrencyId, u32) {
	let id = set_ico::<T>();
	assert_ok!(Ico::<T>::permit_ico(T::PermitIcoOrigin::successful_origin(), id));
	assert_ok!(Ico::<T>::join(RawOrigin::Signed(get_haha::<T>()).into(), id, 1, (200 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), None));
	(id, 1)
}

fn release_requests<T: Config>() -> (CurrencyId, u32) {
	let (id, index) = get_ico::<T>();
	let alice = get_alice::<T>();
	assert_ok!(Ico::<T>::request_release(RawOrigin::Signed(alice.clone()).into(), id, index, Percent::from_percent(10u8)));
	(id, index)
}

fn release_permit<T: Config>() -> (CurrencyId, u32) {
	let (id, index) = release_requests::<T>();
	let alice = get_alice::<T>();
	assert_ok!(Ico::<T>::permit_release(T::PermitReleaseOrigin::successful_origin(), id, index));
	(id, index)
}

fn haha_get_release_amount<T: Config>() -> (T::AccountId, CurrencyId, u32) {
	let (id, index) = release_permit::<T>();
	let haha = get_haha::<T>();
	assert_ok!(Ico::<T>::user_release_ico_amount(RawOrigin::Signed(haha.clone()).into(), id, index));
	(haha, id, index)
}

benchmarks! {
	initiate_ico {
		let alice: T::AccountId = get_alice::<T>();
		let currency_id = T::UsdtCurrencyId::get();
		T::CurrenciesHandler::do_create(alice.clone(), currency_id, Some(DicoAssetMetadata {
			name: vec![1; 4],
			symbol: vec![1; 4],
			decimals: 12
		}), (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), false);
		let bob: T::AccountId = get_bob::<T>();
		T::MultiCurrency::deposit(currency_id, &bob, 10000u128.saturated_into::<MultiBalanceOf<T>>());
		let ico_info = IcoParameters {
			desc: vec![],
			currency_id: T::GetNativeCurrencyId::get(),
			official_website: vec![],
			is_must_kyc: false,
			user_ico_max_times: 2,
			total_issuance: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			total_circulation: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			ico_duration: T::BlockNumber::from(100u32),
			total_ico_amount: (1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			user_min_amount: (100 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			user_max_amount: (500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			exchange_token: currency_id,
			exchange_token_total_amount: (1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			exclude_area: vec![],
			lock_proportion: Default::default(),
			unlock_duration: T::BlockNumber::from(0u32),
			per_duration_unlock_amount: MultiBalanceOf::<T>::from(0u32),
		};
	}:_(RawOrigin::Signed(alice.clone()), ico_info)

	permit_ico {
		let id = set_ico::<T>();
	}: _<T::Origin>(T::PermitIcoOrigin::successful_origin(), id)
	verify {

	}

	reject_ico {
		let id = set_ico::<T>();
	}: _<T::Origin>(T::RejectIcoOrigin::successful_origin(), id)
	verify {

	}

	join {
		let (id, index) = get_ico::<T>();
		let bob = get_bob::<T>();

	}:_(RawOrigin::Signed(bob.clone()), id, index, (200 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), None)

	terminate_ico {
		let (id, index) = get_ico::<T>();
	}:_<T::Origin>(T::TerminateIcoOrigin::successful_origin(), id, index)
	verify {

	}

	request_release {
		let (id, index) = get_ico::<T>();
		let alice = get_alice::<T>();
	}:_(RawOrigin::Signed(alice.clone()), id, index, Percent::from_percent(20u8))

	cancel_request {
		let (id, index) = release_requests::<T>();
		let alice = get_alice::<T>();
	}:_(RawOrigin::Signed(alice.clone()), id, index)

	permit_release {
		let (id, index) = release_requests::<T>();
	}:_<T::Origin>(T::PermitReleaseOrigin::successful_origin(), id, index)
	verify {

	}

	user_release_ico_amount {
		let (id, index) = release_permit::<T>();
		let haha = get_haha::<T>();

	}:_(RawOrigin::Signed(haha.clone()), id, index)

	unlock {
		let (haha, id, index) = haha_get_release_amount::<T>();
	}:_(RawOrigin::Signed(haha.clone()), id, index)

	set_system_ico_amount_bound {

	}:_(RawOrigin::Root, (200 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), (500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>())

	initiator_set_ico_amount_bound {
		let alice = get_alice::<T>();
		let (id, index) = get_ico::<T>();

	}:_(RawOrigin::Signed(alice.clone()), id, index, (200 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(), (500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>())

	initiator_set_ico_max_times {
		let alice = get_alice::<T>();
		let (id, index) = get_ico::<T>();
	}:_(RawOrigin::Signed(alice.clone()), id, index, 2u8)

	get_reward {
		let haha = get_haha::<T>();
		let (id, index) = get_ico::<T>();
	}:_(RawOrigin::Signed(haha.clone()), id, index)

	set_asset_power_multiple {

	}:_(RawOrigin::Root, T::UsdtCurrencyId::get(), PowerMultiple {
		up: 10,
		down: 2,
	})
}

impl_benchmark_test_suite!(Ico, crate::mock::new_test_ext(), crate::mock::Test,);