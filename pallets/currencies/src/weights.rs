
//! Autogenerated weights for `pallet_currencies`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-27, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kico"), DB CACHE: 128

// Executed Command:
// ./target/release/dico
// benchmark
// --execution
// Wasm
// --wasm-execution
// compiled
// --pallet
// pallet_currencies
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw
// --output
// ./pallets/currencies/src/weights.rs
// --chain
// kico

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_currencies`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_currencies::WeightInfo for WeightInfo<T> {
	// Storage: Currencies DicoAssetsInfo (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:1 w:1)
	fn create_asset() -> Weight {
		(45_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Currencies DicoAssetsInfo (r:1 w:1)
	fn set_metadata() -> Weight {
		(15_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Currencies DicoAssetsInfo (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn burn() -> Weight {
		(24_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Currencies DicoAssetsInfo (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	fn transfer() -> Weight {
		(41_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	fn transfer_native_currency() -> Weight {
		(29_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Currencies DicoAssetsInfo (r:1 w:0)
	// Storage: Tokens Accounts (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	fn update_balance() -> Weight {
		(22_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
