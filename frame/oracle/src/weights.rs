#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for `pallet_tidefi`.
pub trait WeightInfo {
   fn set_status() -> Weight;
   fn set_account_id() -> Weight;
   fn confirm_swap() -> Weight;
   fn add_market_maker() -> Weight;
   fn remove_market_maker() -> Weight;
   fn im_alive() -> Weight;
}

/// Weights for `pallet_tidefi` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
   fn set_status() -> Weight {
      61_000_300_u64
          .saturating_add(T::DbWeight::get().reads(6_u64))
          .saturating_add(T::DbWeight::get().writes(5_u64))
  }
  fn add_market_maker() -> Weight {
      61_000_300_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn remove_market_maker() -> Weight {
      61_000_300_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
  fn set_account_id() -> Weight {
      62_000_300_u64
       .saturating_add(T::DbWeight::get().reads(6_u64))
       .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn confirm_swap() -> Weight {
      63_000_400_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
   fn im_alive() -> Weight {
      64_000_400_u64
         .saturating_add(T::DbWeight::get().reads(6_u64))
         .saturating_add(T::DbWeight::get().writes(5_u64))
   }
}