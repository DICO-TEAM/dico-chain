use super::*;
use frame_support::traits::Get;
use pallet_currencies::currencies_trait::AssetIdMapping;
use sp_std::{marker::PhantomData, result::Result};
use xcm_executor::{traits::WeightTrader, Assets};

pub struct FixedRateOfAsset<
	T: Get<u128>,
	R: TakeRevenue,
	M: AssetIdMapping<CurrencyId, MultiLocation>,
>(Weight, u128, Option<MultiLocation>, PhantomData<(T, R, M)>);

impl<T: Get<u128>, R: TakeRevenue, M: AssetIdMapping<CurrencyId, MultiLocation>> WeightTrader
	for FixedRateOfAsset<T, R, M>
{
	fn new() -> Self {
		Self(Weight::from_ref_time(0), 0, None, PhantomData)
	}

	fn buy_weight(&mut self, weight: u64, payment: Assets) -> Result<Assets, XcmError> {
		let payment_cp = payment.clone();
		let asset_id: &AssetId = payment
			.fungible
			.iter()
			.next()
			.map_or(Err(XcmError::TooExpensive), |v| Ok(v.0))?;

		if let Concrete(ref multi_location) = asset_id {
			let units_per_second = T::get();
			let multiple = M::get_weight_rate_multiple(multi_location.clone())
				.ok_or(XcmError::AssetNotFound)?;
			let amount = units_per_second * multiple * (weight as u128) /
				(WEIGHT_PER_SECOND.ref_time() as u128);
			if amount == 0 {
				return Ok(payment)
			}
			let unused = payment_cp
				.checked_sub((asset_id.clone(), amount).into())
				.map_err(|_| XcmError::TooExpensive)?;

			self.0 = self.0.saturating_add(Weight::from_ref_time(weight));
			self.1 = self.1.saturating_add(amount);
			self.2 = Some(multi_location.clone());
			return Ok(unused)
		}
		Err(XcmError::TooExpensive)
	}

	fn refund_weight(&mut self, weight: u64) -> Option<MultiAsset> {
		log::trace!(target: "xcm::weight", "FixedRateOfAsset::refund_weight weight: {:?}", weight);
		match &self.2 {
			Some(l) =>
				if let Some(multiple) = M::get_weight_rate_multiple(l.clone()) {
					let units_per_second = T::get();
					let weight = weight.min(self.0.ref_time());
					let amount = units_per_second * multiple * (weight as u128) /
						(WEIGHT_PER_SECOND.ref_time() as u128);
					self.0 -= Weight::from_ref_time(weight);
					self.1 = self.1.saturating_sub(amount);
					if amount > 0 {
						Some((Concrete(l.clone()), amount).into())
					} else {
						None
					}
				} else {
					None
				},
			None => None,
		}
	}
}

impl<T: Get<u128>, R: TakeRevenue, M: AssetIdMapping<CurrencyId, MultiLocation>> Drop
	for FixedRateOfAsset<T, R, M>
{
	fn drop(&mut self) {
		if self.1 > 0 {
			R::take_revenue((self.2.clone().unwrap(), self.1).into());
		}
	}
}
