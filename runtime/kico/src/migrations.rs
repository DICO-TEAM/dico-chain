use super::*;
pub struct KYCMigrationV2;

impl OnRuntimeUpgrade for KYCMigrationV2 {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		pallet_kyc::migrations::v2::migrate::<Runtime>()
	}

	#[cfg(feature = "try-rutime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		pallet_kyc::migrations::v2::pre_migrate::<Runtime>()
	}

	#[cfg(feature = "try-rntime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_kyc::migrations::v2::post_migrate::<Runtime>()
	}
}
