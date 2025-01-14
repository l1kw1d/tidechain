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

use crate::{
  mock::{new_test_ext, Adapter, Origin, Security, System, Test, TidefiStaking},
  Error,
};
use sp_runtime::Percent;
use tidefi_primitives::BlockNumber;

use frame_support::{
  assert_noop, assert_ok,
  traits::{
    fungibles::{Inspect, Mutate},
    Hooks,
  },
};
use tidefi_primitives::{pallet::StakingExt, CurrencyId};

const TEST_TOKEN: u32 = 2;
const FIFTEEN_DAYS: BlockNumber = 14400 * 15;
const BLOCKS_FORCE_UNLOCK: BlockNumber = 256;

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert_eq!(
      TidefiStaking::staking_rewards()
        .into_iter()
        .find(|(duration, _)| *duration == 14400 * 15),
      Some((14400 * 15, Percent::from_parts(2)))
    );
  });
}

#[test]
pub fn should_stake_wrapped_asset() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &alice, 1_000_000_000_000)
      .expect("Unable to mint token");

    assert_ok!(TidefiStaking::stake(
      alice_origin,
      CurrencyId::Wrapped(TEST_TOKEN),
      // maximum amount
      500_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Wrapped(TEST_TOKEN)),
      Some(500_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == 500_000_000
    );
  });
}

#[test]
pub fn should_fails_amount_too_small() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Wrapped(TEST_TOKEN), &alice, 1_000_000_000_000)
      .expect("Unable to mint token");

    assert_noop!(
      TidefiStaking::stake(
        alice_origin,
        CurrencyId::Wrapped(TEST_TOKEN),
        10,
        FIFTEEN_DAYS
      ),
      Error::<Test>::AmountTooSmall,
    );
  });
}

#[test]
pub fn should_stake_native_asset() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_ok!(TidefiStaking::stake(
      alice_origin,
      CurrencyId::Tifi,
      1_000_000_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(1_000_000_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == 1_000_000_000_000
    );
  });
}

#[test]
pub fn should_stake_and_unstake() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      1_000_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tifi,
      1_000_000_000_000,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      1_000_000_000_000_000 - 1_000_000_000_000
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(1_000_000_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == 1_000_000_000_000
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS + 1;
      *n
    });

    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS + 1);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, false));
    assert!(TidefiStaking::account_stakes(alice).len() == 0);
    // balance is returned
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      1_000_000_000_000_000
    );
  });
}

#[test]
pub fn should_stake_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);
    let initial_stake = 1_000_000_000_000;
    let initial_mint = 1_000_000_000_000_000;

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, initial_mint).expect("Unable to mint token");

    assert_eq!(Adapter::balance(CurrencyId::Tifi, &1u64), initial_mint);

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tifi,
      initial_stake,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      initial_mint - initial_stake
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(initial_stake)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == initial_stake
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 1_000;
      *n
    });

    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 1_000);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, true));
    assert!(TidefiStaking::account_stakes(alice).len() == 1);

    let unstake_fee = TidefiStaking::unstake_fee() * initial_stake;
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      initial_mint - initial_stake - unstake_fee
    );
    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1;
      *n
    });
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 1_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert!(TidefiStaking::unstake_queue().len() == 1);

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert!(TidefiStaking::unstake_queue().len() == 0);
  });
}

#[test]
pub fn should_stake_multiple_and_unstake_queue() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    let bob = 2u64;
    let bob_origin = Origin::signed(bob);

    let initial_stake = 1_000_000_000_000;
    let initial_stake_bob = initial_stake / 4;
    let initial_mint = 1_000_000_000_000_000;

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, initial_mint).expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tifi, &bob, initial_mint).expect("Unable to mint token");

    assert_eq!(Adapter::balance(CurrencyId::Tifi, &alice), initial_mint);
    assert_eq!(Adapter::balance(CurrencyId::Tifi, &bob), initial_mint);

    assert_ok!(TidefiStaking::stake(
      alice_origin.clone(),
      CurrencyId::Tifi,
      initial_stake,
      FIFTEEN_DAYS
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      initial_mint - initial_stake
    );

    let stake_id = TidefiStaking::account_stakes(alice)
      .first()
      .unwrap()
      .unique_id;

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(initial_stake)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance
        == initial_stake
    );

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 3_000;
      *n
    });
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 3_000);

    assert_ok!(TidefiStaking::stake(
      bob_origin.clone(),
      CurrencyId::Tifi,
      initial_stake_bob,
      FIFTEEN_DAYS
    ));

    let bob_stake_id = TidefiStaking::account_stakes(bob)
      .first()
      .unwrap()
      .unique_id;

    assert_ok!(TidefiStaking::stake(
      bob_origin.clone(),
      CurrencyId::Tifi,
      initial_stake_bob,
      FIFTEEN_DAYS * 2
    ));

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000;
      *n
    });
    assert_eq!(Security::current_block_number(), FIFTEEN_DAYS - 2_000);

    assert_ok!(TidefiStaking::unstake(bob_origin, bob_stake_id, true));

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2);
      *n
    });
    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + (BLOCKS_FORCE_UNLOCK / 2)
    );
    assert_eq!(TidefiStaking::unstake_queue().len(), 1);
    assert_ok!(TidefiStaking::unstake(alice_origin, stake_id, true));
    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    assert_eq!(TidefiStaking::account_stakes(alice).len(), 1);
    assert_eq!(TidefiStaking::account_stakes(bob).len(), 2);

    let unstake_fee = TidefiStaking::unstake_fee() * initial_stake;
    let unstake_fee_bob = TidefiStaking::unstake_fee() * initial_stake_bob;

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      initial_mint - initial_stake - unstake_fee
    );

    // 1 % of 1_000_000_000_000 = 10_000_000_000
    assert_eq!(unstake_fee, 10_000_000_000);

    // BlocksForceUnstake is set to 10, so let skip at least 10 blocks
    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1;
      *n
    });

    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + 1
    );

    assert_eq!(TidefiStaking::unstake_queue().len(), 2);

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(TidefiStaking::unstake_queue().len(), 1);

    <pallet_security::CurrentBlockCount<Test>>::mutate(|n| {
      *n = FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1;
      *n
    });

    assert_eq!(
      Security::current_block_number(),
      FIFTEEN_DAYS - 2_000 + BLOCKS_FORCE_UNLOCK + (BLOCKS_FORCE_UNLOCK / 2) + 1
    );
    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert!(TidefiStaking::unstake_queue().is_empty());

    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &1u64),
      initial_mint - unstake_fee
    );

    assert!(TidefiStaking::account_stakes(bob).len() == 1);
    assert_eq!(
      Adapter::balance(CurrencyId::Tifi, &2u64),
      // we still have a stake active
      initial_mint - unstake_fee_bob - initial_stake_bob
    );
  });
}

#[test]
pub fn stake_with_invalid_duration() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_noop!(
      TidefiStaking::stake(alice_origin, CurrencyId::Tifi, 1_000_000_000_000, 999),
      Error::<Test>::InvalidDuration
    );
  });
}

#[test]
pub fn should_calculate_rewards() {
  new_test_ext().execute_with(|| {
    let alice = 1u64;
    let alice_origin = Origin::signed(alice);
    let bob = 2u64;
    let bob_origin = Origin::signed(bob);
    let charlie = 3u64;
    let charlie_origin = Origin::signed(charlie);

    System::set_block_number(1);

    // mint token to user
    Adapter::mint_into(CurrencyId::Tifi, &alice, 1_000_000_000_000_000)
      .expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tifi, &bob, 1_000_000_000_000_000)
      .expect("Unable to mint token");
    Adapter::mint_into(CurrencyId::Tifi, &charlie, 1_000_000_000_000_000)
      .expect("Unable to mint token");

    assert_ok!(TidefiStaking::stake(
      alice_origin,
      CurrencyId::Tifi,
      100_000_000_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(100_000_000_000_000)
    );

    // make sure the staking has been recorded in the storage
    assert!(TidefiStaking::account_stakes(alice).len() == 1);
    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .initial_balance,
      100_000_000_000_000
    );

    // 100 for TIFI in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      1,
      vec![(CurrencyId::Tifi, 100_000_000_000_000)]
    ));

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    // started with 100, now should have 102 tides
    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      102_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      bob_origin,
      CurrencyId::Tifi,
      100_000_000_000_000,
      FIFTEEN_DAYS
    ));

    // make sure the staking pool has been updated
    assert_eq!(
      TidefiStaking::staking_pool(CurrencyId::Tifi),
      Some(200_000_000_000_000)
    );

    // 100 for TIFI in fees for session 1
    // 15 days should get 2%, so 2 tides
    assert_ok!(TidefiStaking::on_session_end(
      2,
      vec![(CurrencyId::Tifi, 100_000_000_000_000)]
    ));

    // run on idle hook
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_000_000_000_000
    );

    // 2 empty sessions
    assert_ok!(TidefiStaking::on_session_end(3, Vec::new()));
    assert_ok!(TidefiStaking::on_session_end(4, Vec::new()));
    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_000_000_000_000
    );

    assert_ok!(TidefiStaking::stake(
      charlie_origin,
      CurrencyId::Tifi,
      400_000_000_000_000,
      FIFTEEN_DAYS
    ));

    assert_ok!(TidefiStaking::on_session_end(
      5,
      vec![(CurrencyId::Tifi, 100_000_000_000_000)]
    ));

    assert_eq!(
      TidefiStaking::on_idle(1, 1_000_000_000_000_000),
      1_000_000_000_000_000
    );

    assert_eq!(
      TidefiStaking::account_stakes(alice)
        .first()
        .unwrap()
        .principal,
      103_333_333_333_333
    );

    assert_eq!(
      TidefiStaking::account_stakes(bob)
        .first()
        .unwrap()
        .principal,
      101_333_333_333_333
    );

    assert_eq!(
      TidefiStaking::account_stakes(charlie)
        .first()
        .unwrap()
        .principal,
      401_333_333_333_333
    );
  });
}
