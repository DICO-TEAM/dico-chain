#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight as DbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_prices.
pub trait WeightInfo {
    fn lock_price() -> Weight;
    fn unlock_price() -> Weight;
    fn del_feed_account(c: u32) -> Weight;
    fn insert_feed_account(c: u32) -> Weight;
    fn withdraw() -> Weight;
    fn exit_feed() -> Weight;
}

impl  WeightInfo for () {
    fn lock_price() -> Weight {
        (70_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }

    fn unlock_price() -> Weight {
        (12_000_000 as Weight)
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }

    fn del_feed_account(c: u32) -> Weight {
        (12_000_000 as Weight)
            .saturating_add(DbWeight::get().writes(1 as Weight).saturating_mul(c as Weight))
    }

    fn insert_feed_account(c: u32) -> Weight {
        (10_000_000 as Weight)
            .saturating_add(DbWeight::get().writes(1 as Weight).saturating_mul(c as Weight))
    }

    fn exit_feed() -> Weight {
        (70_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }

    fn withdraw() -> Weight {
        (70_000_000 as Weight)
            .saturating_add(DbWeight::get().reads(11 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }


}
