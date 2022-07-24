#![allow(unused_must_use)]

use frame_support::traits::{Currency, WithdrawReasons, LockableCurrency};
use frame_support::traits::fungible::Mutate;
use sp_runtime::BoundedVec;
use super::*;

parameter_types! {
	// 5EKzRRVjvBvZfcRJPaHJCw2yecP9uQXcm6vqNcnMh6bCjpPe
	pub OF: AccountId = hex_literal::hex!["6420e8ea68ad42811f49e22dea051ed55b2ec7ee5f193dc38fbec426422fc65a"].into();
	// 5Df8Wohqqo5zYbiPgJ6teaU8n3GyH5o9E5YMnuUXwbqDTTyo
	pub NF: AccountId = hex_literal::hex!["467d94f60ca2fec8bdfc843926088df5ad274feb4a8c2cd0465ca8cb78f54c72"].into();
	// 5HWKdjaQFe1jFfSVe215GKjMeExe4kVx9iK3SQW2eAZg4vJs
	pub OP: AccountId = hex_literal::hex!["f0b6bd0e4ba33cc8670103826e53111326f3d4c6df55d32b03701ef8be22ad4a"].into();
	// 5F6oqQ54oq8V5ZdUA251ZuCw4ztc9RHzCmc9pGMe4KC491RV
	pub NP: AccountId = hex_literal::hex!["864f50b411342f78661932fdcc1382f6b47708033c4e42dc20a723e08201ff54"].into();
}

pub struct ResetTechnicalCommitteeMember;

impl OnRuntimeUpgrade for ResetTechnicalCommitteeMember {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut old_members: Vec<AccountId>= pallet_collective::Members::<Runtime, TechnicalCollective>::get();
		old_members.retain(|who| who != &OF::get());
		old_members.retain(|who| who != &OP::get());
		pallet_collective::Members::<Runtime, TechnicalCollective>::put(old_members);

		pallet_collective::Prime::<Runtime, TechnicalCollective>::put(NP::get());
		RocksDbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert!(pallet_collective::Members::<Runtime, TechnicalCollective>::get().contains(&OP::get()), "old members not exists");
		assert!(pallet_collective::Prime::<Runtime, TechnicalCollective>::get() == Some(OP::get()), "not old prime");
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert!(pallet_collective::Prime::<Runtime, TechnicalCollective>::get() == Some(NP::get()), "set new prime not success");
		assert!(!pallet_collective::Members::<Runtime, TechnicalCollective>::get().contains(&OP::get()), "remove old members not success");
		Ok(())
	}
}

pub struct MigrationOPVesting;

impl OnRuntimeUpgrade for MigrationOPVesting {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let of_free_amount: Balance = pallet_balances::Pallet::<Runtime>::total_balance(&OF::get());
		pallet_balances::Pallet::<Runtime>::burn_from(&OF::get(), of_free_amount);
		frame_system::Account::<Runtime>::take(&OF::get());
		pallet_balances::Pallet::<Runtime>::mint_into(&NF::get(), of_free_amount);

		let mut of_vesting_info = orml_vesting::VestingSchedules::<Runtime>::take(&OF::get()).into_inner();
		let misc = 36812336377456430100000u128 + 1724099999999999901696u128;
		let lock_id = orml_vesting::VESTING_LOCK_ID;
		orml_vesting::VestingSchedules::<Runtime>::mutate(&NF::get(), |f| f.try_append(&mut of_vesting_info));
		Balances::set_lock(lock_id, &NF::get(), misc, WithdrawReasons::all());

		RocksDbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert!(pallet_balances::Pallet::<Runtime>::free_balance(&OF::get()) > 3_5000_0000 * DOLLARS, "old fund amount not enough");
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert!(pallet_balances::Pallet::<Runtime>::free_balance(&OF::get()).is_zero(), "old fund frre amount is not zero");
		assert!(orml_vesting::VestingSchedules::<Runtime>::get(&OF::get()).len() == 0, "old fund vesting info exists");
		assert!(orml_vesting::VestingSchedules::<Runtime>::get(&NF::get()).len() >= 2, "new fund vesting info err");

		Ok(())
	}
}
