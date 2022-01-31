//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kico"), DB CACHE: 128

// Executed Command:
// target/release/dico
// benchmark
// --chain=kico
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_multisig
// --extrinsic=*
// --steps=50
// --repeat=20
// --raw
// --output=./runtime/kico/src/weights/pallet_multisig.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
    fn as_multi_threshold_1(z: u32, ) -> Weight {
        (21_500_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create(s: u32, z: u32, ) -> Weight {
        (52_795_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((178_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create_store(s: u32, z: u32, ) -> Weight {
        (61_376_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((165_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    fn as_multi_approve(s: u32, z: u32, ) -> Weight {
        (33_705_000 as Weight)
            // Standard Error: 0
            .saturating_add((164_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:1)
    fn as_multi_approve_store(s: u32, z: u32, ) -> Weight {
        (55_990_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((173_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn as_multi_complete(s: u32, z: u32, ) -> Weight {
        (71_501_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((268_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn approve_as_multi_create(s: u32, ) -> Weight {
        (50_406_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((197_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:0)
    fn approve_as_multi_approve(s: u32, ) -> Weight {
        (29_530_000 as Weight)
            // Standard Error: 0
            .saturating_add((193_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn approve_as_multi_complete(s: u32, ) -> Weight {
        (94_116_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((308_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Multisig Multisigs (r:1 w:1)
    // Storage: Multisig Calls (r:1 w:1)
    fn cancel_as_multi(s: u32, ) -> Weight {
        (78_102_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((185_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}