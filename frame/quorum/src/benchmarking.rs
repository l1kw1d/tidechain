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

#![cfg(feature = "runtime-benchmarks")]
use super::*;

use frame_benchmarking::{
  account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec,
};
use frame_support::BoundedVec;
use frame_system::{self, RawOrigin};
use tidefi_primitives::{
  pallet::SecurityExt, ComplianceLevel, CurrencyId, Hash, Mint, ProposalType, ProposalVotes,
};

const SEED: u32 = 0;
const ADMIN_ID: u32 = 1;

fn _assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
  frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn pre_set_auth<T: Config>() -> T::AccountId {
  let user: T::AccountId = account("admin", ADMIN_ID, SEED);
  Members::<T>::remove_all();
  Members::<T>::insert(&user, true);
  let public_key: BoundedVec<u8, <T as pallet::Config>::StringLimit> =
    "pubkey".as_bytes().to_vec().try_into().unwrap();
  let public_keys: BoundedVec<
    (
      T::AccountId,
      BoundedVec<u8, <T as pallet::Config>::StringLimit>,
    ),
    <T as pallet::Config>::PubkeyLimitPerAsset,
  > = vec![(user.clone(), public_key)].try_into().unwrap();

  PublicKeys::<T>::insert(1, public_keys);

  Threshold::<T>::put(1);
  user
}

fn create_proposal<T: Config>() -> Hash {
  let account_id: T::AccountId = whitelisted_caller();
  let proposal = ProposalType::Mint(Mint {
    account_id,
    currency_id: CurrencyId::Tifi,
    mint_amount: 1_000_000_000_000,
    transaction_id: Default::default(),
    compliance_level: ComplianceLevel::Green,
  });

  let proposal_id = Hash::zero();
  Proposals::<T>::try_append((
    proposal_id,
    T::Security::get_current_block_count(),
    proposal,
  ))
  .unwrap();

  proposal_id
}

benchmarks! {
   submit_proposal {
      let user = pre_set_auth::<T>();
      let account_id: T::AccountId = whitelisted_caller();
      let proposal = ProposalType::Mint(Mint {
         account_id,
         currency_id: CurrencyId::Tifi,
         mint_amount: 1_000_000_000_000,
         transaction_id: Vec::new(),
         compliance_level: ComplianceLevel::Green,
      });
   }: _(RawOrigin::Signed(user), proposal)
   acknowledge_proposal {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
   }: _(RawOrigin::Signed(user), proposal_id)
   reject_proposal {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
   }: _(RawOrigin::Signed(user), proposal_id)
   eval_proposal_state {
      let user = pre_set_auth::<T>();
      let proposal_id = create_proposal::<T>();
      Votes::<T>::insert(proposal_id, ProposalVotes::default());
   }: _(RawOrigin::Signed(user), proposal_id)
   submit_public_keys {
      // test with 100 assets
      let k in 0 .. 100;
      let mut all_keys = vec![];
      for i in 0..k {
         all_keys.push((i as u32, "new_pubkey".as_bytes().to_vec()));
      }
      let user = pre_set_auth::<T>();
   }: _(RawOrigin::Signed(user), all_keys)
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
