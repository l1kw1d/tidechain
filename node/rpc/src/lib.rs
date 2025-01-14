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

use std::sync::Arc;

use sc_client_api::AuxStore;
use sc_consensus_babe::{Config, Epoch};
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_consensus_epochs::SharedEpochChanges;
use sc_finality_grandpa::{
  FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::SyncCryptoStorePtr;
use tidefi_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Index};

pub use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
  /// BABE protocol config.
  pub babe_config: Config,
  /// BABE pending epoch changes.
  pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
  /// The keystore that manages the keys of the node.
  pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
  /// Voting round info.
  pub shared_voter_state: SharedVoterState,
  /// Authority set info.
  pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
  /// Receives notifications about justification events from Grandpa.
  pub justification_stream: GrandpaJustificationStream<Block>,
  /// Executor to drive the subscription manager in the Grandpa RPC handler.
  pub subscription_executor: SubscriptionTaskExecutor,
  /// Finality proof provider.
  pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
  /// The client instance to use.
  pub client: Arc<C>,
  /// Transaction pool instance.
  pub pool: Arc<P>,
  /// The `SelectChain` Strategy
  pub select_chain: SC,
  /// A copy of the chain spec.
  pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
  /// Whether to deny unsafe calls
  pub deny_unsafe: DenyUnsafe,
  /// BABE specific dependencies.
  pub babe: BabeDeps,
  /// GRANDPA specific dependencies.
  pub grandpa: GrandpaDeps<B>,
}

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, B>(
  deps: FullDeps<C, P, SC, B>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
  C: ProvideRuntimeApi<Block>
    + HeaderBackend<Block>
    + AuxStore
    + HeaderMetadata<Block, Error = BlockChainError>
    + Sync
    + Send
    + 'static,
  C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
  C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
  C::Api: BabeApi<Block>,
  C::Api: BlockBuilder<Block>,
  // Tidechain API
  C::Api: pallet_tidefi_rpc::TidefiRuntimeApi<Block, AccountId>,
  P: TransactionPool + 'static,
  SC: SelectChain<Block> + 'static,
  B: sc_client_api::Backend<Block> + Send + Sync + 'static,
  B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
  use pallet_tidefi_rpc::{Tidefi, TidefiApi};
  use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
  use substrate_frame_rpc_system::{FullSystem, SystemApi};

  let mut io = jsonrpc_core::IoHandler::default();
  let FullDeps {
    client,
    pool,
    select_chain,
    chain_spec,
    deny_unsafe,
    babe,
    grandpa,
  } = deps;

  let BabeDeps {
    keystore,
    babe_config,
    shared_epoch_changes,
  } = babe;

  let GrandpaDeps {
    shared_voter_state,
    shared_authority_set,
    justification_stream,
    subscription_executor,
    finality_provider,
  } = grandpa;

  io.extend_with(SystemApi::to_delegate(FullSystem::new(
    client.clone(),
    pool,
    deny_unsafe,
  )));

  io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
    client.clone(),
  )));

  io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
    BabeRpcHandler::new(
      client.clone(),
      shared_epoch_changes.clone(),
      keystore,
      babe_config,
      select_chain,
      deny_unsafe,
    ),
  ));

  io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
    GrandpaRpcHandler::new(
      shared_authority_set.clone(),
      shared_voter_state,
      justification_stream,
      subscription_executor,
      finality_provider,
    ),
  ));

  io.extend_with(sc_sync_state_rpc::SyncStateRpcApi::to_delegate(
    sc_sync_state_rpc::SyncStateRpcHandler::new(
      chain_spec,
      client.clone(),
      shared_authority_set,
      shared_epoch_changes,
    )?,
  ));

  // Tidechain Custom traits
  io.extend_with(TidefiApi::to_delegate(Tidefi::new(client)));

  Ok(io)
}
