#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{
	account, benchmarks, benchmarks_instance, benchmarks_instance_pallet, impl_benchmark_test_suite, whitelist_account,
	whitelisted_caller,
};
use frame_support::assert_ok;
use sp_runtime::SaturatedConversion;

use crate::Pallet as NFT;
use dico_primitives::currency::DOLLARS;
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn get_bob<T: Config>() -> T::AccountId {
	let Bob = account("bob", 2, SEED);
	T::Currency::make_free_balance_be(&Bob, (10000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	Bob
}

fn create_nft_class<T: Config>() -> (T::AccountId, T::ClassId) {
	let caller: T::AccountId = whitelisted_caller();
	T::Currency::make_free_balance_be(&caller, (10000 * DOLLARS).saturated_into::<BalanceOf<T>>());
	let remark_message = vec![1; 3];
	let class = ClassData {
		level: NftLevel::Rookie,
		power_threshold: BalanceOf::<T>::default(),
		claim_payment: BalanceOf::<T>::default(),
		images_hash: None,
		maximum_quantity: 100u32.into(),
	};
	assert!(NFT::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), remark_message, class).is_ok());
	(caller, T::ClassId::default())
}

fn create_nft_token<T: Config>() -> (T::AccountId, T::ClassId, T::TokenId) {
	let (caller, class_id) = create_nft_class::<T>();
	let string = vec![1; 3];
	assert!(NFT::<T>::mint(
		RawOrigin::Signed(caller.clone()).into(),
		class_id,
		string.clone(),
		string.clone(),
		string
	)
	.is_ok());
	(caller, class_id, T::TokenId::default())
}

fn claim_nft_token<T: Config>() -> (T::AccountId, T::ClassId, T::TokenId) {
	let (caller, class_id, token_id) = create_nft_token::<T>();
	assert!(NFT::<T>::claim(RawOrigin::Signed(caller.clone()).into(), (class_id, token_id)).is_ok());
	(caller, class_id, T::TokenId::default())
}

fn sale_token<T: Config>() -> (T::ClassId, T::TokenId) {
	let (caller, class_id, token_id) = claim_nft_token::<T>();
	assert!(NFT::<T>::offer_token_for_sale(
		RawOrigin::Signed(caller.clone()).into(),
		(class_id, token_id),
		BalanceOf::<T>::from(20u32)
	)
	.is_ok());
	(class_id, token_id)
}

fn active_nft<T: Config>() -> (T::AccountId, T::ClassId, T::TokenId) {
	let (caller, class_id, token_id) = claim_nft_token::<T>();
	assert!(NFT::<T>::active(RawOrigin::Signed(caller.clone()).into(), (class_id, token_id)).is_ok());
	(caller, class_id, token_id)
}

benchmarks! {
	create_class {
		let caller: T::AccountId = whitelisted_caller();
		let remark_message = vec![1; 3];

	}:_(RawOrigin::Signed(caller.clone()), remark_message, ClassData {
		level: NftLevel::Rookie,
		power_threshold: Default::default(),
		claim_payment: Default::default(),
		images_hash: None,
		maximum_quantity: Default::default(),
	})
	verify {
		assert_eq!(Classes::<T>::contains_key(T::ClassId::default()), true);
	}

	mint {
		let (caller, class_id) = create_nft_class::<T>();
		let string = vec![1; 3];
	}:_(RawOrigin::Signed(caller.clone()), class_id, string.clone(), string.clone(), string.clone())
	verify {
		assert_eq!(Tokens::<T>::contains_key(class_id, T::TokenId::default()), true);
	}

	claim {
		let (caller, class_id, token_id) = create_nft_token::<T>();
	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && token_info.clone().unwrap().owner.is_some() && token_info.clone().unwrap().owner.unwrap() == caller);
	}

	burn {
		let (caller, class_id, token_id) = claim_nft_token::<T>();

	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && token_info.clone().unwrap().owner.is_none());
	}

	transfer {
		let Bob = get_bob::<T>();
		let (caller, class_id, token_id) = claim_nft_token::<T>();

	}:_(RawOrigin::Signed(caller.clone()), Bob.clone(), (class_id, token_id))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && token_info.clone().unwrap().owner.is_some() && token_info.clone().unwrap().owner.unwrap() == Bob);
	}

	offer_token_for_sale {
		let (caller, class_id, token_id) = claim_nft_token::<T>();

	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id), BalanceOf::<T>::from(20u32))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && token_info.unwrap().data.status.is_in_sale);
	}

	withdraw_sale {
		let (class_id, token_id) = sale_token::<T>();
		let caller: T::AccountId = whitelisted_caller();
	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && !token_info.unwrap().data.status.is_in_sale);
	}

	buy_token {
		let Bob = get_bob::<T>();
		let (class_id, token_id) = sale_token::<T>();

	}:_(RawOrigin::Signed(Bob.clone()), (class_id, token_id))
	verify {
		let token_info = Tokens::<T>::get(class_id, token_id);
		assert!(token_info.is_some() && token_info.clone().unwrap().owner.is_some() && token_info.clone().unwrap().owner.unwrap() == Bob);
	}

	active {
		let (caller, class_id, token_id) = claim_nft_token::<T>();
		// let Bob = get_bob::<T>();
	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id))

	inactive {
		let (caller, class_id, token_id) = active_nft::<T>();

	}:_(RawOrigin::Signed(caller.clone()), (class_id, token_id))
}

// impl_benchmark_test_suite!(NFT, crate::mock::new_test_ext(), crate::mock::Runtime);
#[cfg(test)]
mod test1 {
	use super::*;
	use crate::mock::{new_test_ext, Runtime};
	use frame_support::assert_ok;
	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			// assert_ok!(Currencies::<Runtime>::test_benchmark_set_metadata());
			assert_ok!(NFT::<Runtime>::test_benchmark_inactive());
		});
	}
}
