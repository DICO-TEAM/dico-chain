//! Autogenerated weights for pallet_oracle
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-28, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tico"), DB CACHE: 128

// Executed Command:
// ./target/release/dico
// benchmark
// --chain=tico
// --steps=50
// --repeat=20
// --pallet=pallet_oracle
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=/home/fjy/ttt/oracle_weights.rs
// --template=.maintain/pallet-weight-template.hbs
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_oracle.
pub trait WeightInfo {
	fn feed_values() -> Weight;
}

/// Weights for pallet_oracle using the dico-chain node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	fn feed_values() -> Weight {
		(46_503_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn feed_values() -> Weight {
		(46_503_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
}
