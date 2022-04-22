
// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

//! Autogenerated weights for `frame_system`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("lagoon-dev"), DB CACHE: 128

// Executed Command:
// target/release/tidechain
// benchmark
// --chain=lagoon-dev
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./FILE_TEMPLATE
// --output=./runtime/lagoon/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `frame_election_provider_support`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_election_provider_support::WeightInfo for WeightInfo<T> {
	fn phragmen(v: u32, _t: u32, d: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 51_000
			.saturating_add((18_247_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 7_075_000
			.saturating_add((2_979_831_000 as Weight).saturating_mul(d as Weight))
	}
	fn phragmms(v: u32, _t: u32, d: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 49_000
			.saturating_add((16_045_000 as Weight).saturating_mul(v as Weight))
			// Standard Error: 6_844_000
			.saturating_add((2_712_807_000 as Weight).saturating_mul(d as Weight))
	}
}