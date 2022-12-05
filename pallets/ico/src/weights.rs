//! Autogenerated weights for pallet_ico
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kico"), DB CACHE: 1024

// Executed Command:
// target/release/dico benchmark --chain=kico --execution=wasm --wasm-execution=compiled
// --pallet=pallet_ico --extrinsic=*  --steps=50 --repeat=20
// --template=./.maintain/pallet-weight-template.hbs --output ./pallets/ico/src/weights.rs
//
// #![cfg_attr(rustfmt, rustfmt_skip)]
// #![allow(unused_parens)]
// #![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ico.
pub trait WeightInfo {
	fn initiate_ico() -> Weight;
	fn permit_ico() -> Weight;
	fn reject_ico() -> Weight;
	fn join() -> Weight;
	fn terminate_ico() -> Weight;
	fn request_release() -> Weight;
	fn cancel_request() -> Weight;
	fn permit_release() -> Weight;
	fn user_release_ico_amount() -> Weight;
	fn unlock() -> Weight;
	fn set_system_ico_amount_bound() -> Weight;
	fn initiator_set_ico_amount_bound() -> Weight;
	fn initiator_set_ico_max_times() -> Weight;
	fn get_reward() -> Weight;
	fn set_asset_power_multiple() -> Weight;
}

/// Weights for pallet_ico using the Substrate node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:2 w:0)
	// Storage: Ico TotalNum (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Kyc KYCOf (r:1 w:0)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	fn initiate_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico Indexs (r:1 w:1)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	// Storage: Ico PassedIcoes (r:1 w:1)
	// Storage: Ico Ico (r:0 w:1)
	// Storage: Ico IsUnservePledge (r:0 w:1)
	fn permit_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	fn reject_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:1 w:0)
	// Storage: Ico PowerMultipleOf (r:1 w:0)
	// Storage: Ico UnReleaseAssets (r:2 w:2)
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico IcoesOf (r:1 w:1)
	// Storage: Ico TotalPowerOf (r:1 w:1)
	fn join() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico PassedIcoes (r:1 w:1)
	fn terminate_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn request_release() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn cancel_request() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn permit_release() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico IsUnservePledge (r:1 w:1)
	// Storage: Ico UnReleaseAssets (r:2 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico IcoLocks (r:1 w:1)
	fn user_release_ico_amount() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico IcoLocks (r:1 w:1)
	fn unlock() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico IcoMaxUsdtAmount (r:0 w:1)
	// Storage: Ico IcoMinUsdtAmount (r:0 w:1)
	fn set_system_ico_amount_bound() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	fn initiator_set_ico_amount_bound() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	fn initiator_set_ico_max_times() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico TotalUsdt (r:1 w:1)
	// Storage: Ico UnReleaseAssets (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn get_reward() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PowerMultipleOf (r:1 w:1)
	fn set_asset_power_multiple() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:2 w:0)
	// Storage: Ico TotalNum (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Kyc KYCOf (r:1 w:0)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	fn initiate_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico Indexs (r:1 w:1)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	// Storage: Ico PassedIcoes (r:1 w:1)
	// Storage: Ico Ico (r:0 w:1)
	// Storage: Ico IsUnservePledge (r:0 w:1)
	fn permit_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PendingIco (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Ico InitiatedIcoesOf (r:1 w:1)
	fn reject_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:1 w:0)
	// Storage: Ico PowerMultipleOf (r:1 w:0)
	// Storage: Ico UnReleaseAssets (r:2 w:2)
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico IcoesOf (r:1 w:1)
	// Storage: Ico TotalPowerOf (r:1 w:1)
	fn join() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico PassedIcoes (r:1 w:1)
	fn terminate_ico() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn request_release() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn cancel_request() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico RequestReleaseInfo (r:1 w:1)
	fn permit_release() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:0)
	// Storage: Ico IsUnservePledge (r:1 w:1)
	// Storage: Ico UnReleaseAssets (r:2 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Ico IcoLocks (r:1 w:1)
	fn user_release_ico_amount() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico IcoLocks (r:1 w:1)
	fn unlock() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico IcoMaxUsdtAmount (r:0 w:1)
	// Storage: Ico IcoMinUsdtAmount (r:0 w:1)
	fn set_system_ico_amount_bound() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico IcoMinUsdtAmount (r:1 w:0)
	// Storage: Ico IcoMaxUsdtAmount (r:1 w:0)
	fn initiator_set_ico_amount_bound() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	fn initiator_set_ico_max_times() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico Ico (r:1 w:1)
	// Storage: Ico TotalUsdt (r:1 w:1)
	// Storage: Ico UnReleaseAssets (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn get_reward() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: Ico PowerMultipleOf (r:1 w:1)
	fn set_asset_power_multiple() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}
