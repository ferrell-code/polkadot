// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

/// Money matters.
pub mod currency {
	use primitives::v0::Balance;

	/// The number of balance UNITS per one ROC. 1x10^12
	pub const UNITS_PER_ROC: Balance = 1_000_000_000_000;
	/// Easier to reference this way.
	pub const ROC: Balance = UNITS_PER_ROC;

	/// ROC has no USD value, so we simply say 1 ROC is 1 USD for these configurations.
	/// NOTE: This is written funny to more easily interpret the value of 1 USD per ROC.
	pub const MILLICENTS_PER_ROC: Balance = 1_00_000;

	/// The approximate number of UNITS for one US Dollar and so on...
	pub const MILLICENTS: Balance = UNITS_PER_ROC / MILLICENTS_PER_ROC;
	pub const CENTS: Balance = MILLICENTS * 1000;
	pub const DOLLARS: Balance = CENTS * 100;


	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// Time and blocks.
pub mod time {
	use primitives::v0::{Moment, BlockNumber};
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
	frame_support::parameter_types! {
		pub storage EpochDurationInBlocks: BlockNumber = 10 * MINUTES;
	}

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Size restrictions.
pub mod size {
	pub use primitives::v1::MAX_CODE_SIZE;
}

/// Fee-related.
pub mod fee {
	pub use sp_runtime::Perbill;
	use primitives::v0::Balance;
	use runtime_common::ExtrinsicBaseWeight;
	use frame_support::weights::{
		WeightToFeePolynomial, WeightToFeeCoefficient, WeightToFeeCoefficients,
	};
	use smallvec::smallvec;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, frame_system::MaximumBlockWeight]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Westend, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

#[cfg(test)]
mod tests {
	use frame_support::weights::{WeightToFeePolynomial, DispatchClass};
	use runtime_common::BlockWeights;
	use super::fee::WeightToFee;
	use super::currency::{CENTS, DOLLARS, MILLICENTS};

	#[test]
	// This function tests that the fee for `MaximumBlockWeight` of weight is correct
	fn full_block_fee_is_correct() {
		// A full block should cost 16 DOLLARS
		println!("Base: {}", BlockWeights::get().get(DispatchClass::Normal).base_extrinsic);
		let x = WeightToFee::calc(&BlockWeights::get().max_block);
		let y = 16 * DOLLARS;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}

	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/10 of a CENT
		let base_weight = BlockWeights::get().get(DispatchClass::Normal).base_extrinsic;
		println!("Base: {}", base_weight);
		let x = WeightToFee::calc(&base_weight);
		let y = CENTS / 10;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}
}
