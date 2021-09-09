#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_kyc.
pub trait WeightInfo {
	/// For sudo super-users(Sudo)
	fn add_ias(_r: u32) -> Weight;
	fn add_sword_holder(_r: u32) -> Weight;
	fn remove_kyc(_s: u32) -> Weight;
	fn kill_ias(r: u32) -> Weight;
	fn kill_sword_holder(r: u32) -> Weight;

	/// For sword holder
	fn sword_holder_provide_judgement(_r: u32, _x: u32) -> Weight;
	fn sword_holder_set_fee(r: u32) -> Weight;

	/// identity authentication service(IAS)
	fn ias_set_fee(r: u32) -> Weight;
	fn ias_set_fields(r: u32) -> Weight;
	fn ias_provide_judgement(r: u32, x: u32) -> Weight;
	fn apply_certification(r: u32, x: u32) -> Weight;
	fn ias_request_sword_holder(r: u32, x: u32) -> Weight;

	/// For general users
	fn set_kyc(r: u32, x: u32) -> Weight;
	fn clear_kyc(r: u32, x: u32) -> Weight;
	fn get_ias(r: u32, x: u32) -> Weight;
	fn request_judgement(r: u32, x: u32) -> Weight;
	fn cancel_request(r: u32, x: u32) -> Weight;
}

/// Weights for pallet_kyc using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn add_ias(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn add_sword_holder(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn remove_kyc(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn kill_ias(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn kill_sword_holder(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn sword_holder_provide_judgement(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn sword_holder_set_fee(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn ias_set_fee(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn ias_set_fields(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn ias_provide_judgement(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn apply_certification(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn ias_request_sword_holder(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn set_kyc(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn clear_kyc(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn get_ias(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn request_judgement(r: u32, x: u32) -> Weight {
		(75_299_000 as Weight)
			.saturating_add((493_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((2_014_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}

	fn cancel_request(r: u32, x: u32) -> Weight {
		(67_492_000 as Weight)
			.saturating_add((225_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((2_003_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn add_ias(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn add_sword_holder(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn kill_ias(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn kill_sword_holder(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn remove_kyc(r: u32) -> Weight {
		(28_965_000 as Weight)
			.saturating_add((421_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn set_kyc(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn clear_kyc(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn get_ias(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn apply_certification(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn request_judgement(r: u32, x: u32) -> Weight {
		(75_299_000 as Weight)
			.saturating_add((493_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((2_014_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn cancel_request(r: u32, x: u32) -> Weight {
		(67_492_000 as Weight)
			.saturating_add((225_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((2_003_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn ias_set_fee(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn sword_holder_set_fee(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn ias_set_fields(r: u32) -> Weight {
		(11_375_000 as Weight)
			.saturating_add((382_000 as Weight).saturating_mul(r as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn ias_provide_judgement(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn ias_request_sword_holder(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}

	fn sword_holder_provide_judgement(r: u32, x: u32) -> Weight {
		(71_923_000 as Weight)
			.saturating_add((529_000 as Weight).saturating_mul(r as Weight))
			.saturating_add((1_763_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
