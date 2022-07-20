#![allow(unused_must_use)]
use super::*;

pub struct KYCMigrationV2;

impl OnRuntimeUpgrade for KYCMigrationV2 {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		pallet_kyc::migrations::v2::migrate::<Runtime>()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		pallet_kyc::migrations::v2::pre_migrate::<Runtime>()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_kyc::migrations::v2::post_migrate::<Runtime>()
	}
}

parameter_types! {
	pub KSMAccount: AccountId = hex_literal::hex!["7adc65bdbe79ac3ef6a0c139164b8436e2a81c4431496d5b7616a6f0cecdd367"].into();
	pub AUSDAccount: AccountId = hex_literal::hex!["0824a35ab6895fa2ebe9f0006defcb3e306c67f63c4b22408e30b252661e9822"].into();
}
pub struct DepositAssetsTokens;

impl OnRuntimeUpgrade for DepositAssetsTokens {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let ksm_account: AccountId = KSMAccount::get();
		let ausd_account: AccountId = AUSDAccount::get();
		let ksm_amount: Balance = 2 * DOLLARS;
		let ausd_amount: Balance = 50 * DOLLARS;
		pallet_currencies::Pallet::<Runtime>::deposit(KSM, &ksm_account, ksm_amount);
		pallet_currencies::Pallet::<Runtime>::deposit(AUSD, &ausd_account, ausd_amount);
		RocksDbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}
}
