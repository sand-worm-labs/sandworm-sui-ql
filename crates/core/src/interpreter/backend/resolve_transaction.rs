use super::resolve_checkpoint::{batch_get_checkpoints, get_checkpoint};
use crate::common::{
    chain::ChainOrRpc,
    checkpoint::CheckpointId,
    query_result::TransactionQueryRes,
    transaction::{Transaction, TransactionField},
};
use anyhow::{Ok, Result};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sui_json_rpc_types::{
    SuiTransactionBlockDataAPI, SuiTransactionBlockEffectsAPI,
    SuiTransactionBlockResponse as RpcTransaction, SuiTransactionBlockResponseOptions,
};
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::digests::TransactionDigest;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum TransactionResolverErrors {
    #[error("Mismatch between Entity and EntityId, {0} can't be resolved as a transaction id")]
    MismatchEntityAndEntityId(String),
    #[error("Query should either provide tx hash or block number/range filter")]
    MissingTransactionHashOrFilter,
}

/// Resolve the query to get transactions after receiving an transaction entity expression
/// Iterate through entity_ids and map them to a futures list. Execute all futures concurrently and collect the results.
/// The sequence of steps to fetch transactions is:
/// 1. Check if ids are provided.
/// 2. If ids are provided, fetch the transactions.
/// 3. If ids are not provided, fetch the transactions by block number.
/// 4. If ids are not provided, then block number or block range filter must be provided.
/// 5. Fetch the transactions by block number or block range.
/// 6. If both ids and block number or block range filter are provided, then fetch the transactions by ids first, and filter the result by block number or block range.
pub async fn resolve_transaction_query(
    transaction: &Transaction,
    chains: &[ChainOrRpc],
) -> Result<Vec<TransactionQueryRes>> {
    if !transaction.ids().is_some() && !transaction.has_checkpoint_filter() {
        return Err(TransactionResolverErrors::MissingTransactionHashOrFilter.into());
    }

    let mut all_results = Vec::new();

    for chain in chains {
        let provider = SuiClientBuilder::default().build(chain.rpc_url()?).await?;

        // Fetch transactions for this chain
        let rpc_transactions = match transaction.ids() {
            Some(ids) => get_transactions_by_ids(ids, &provider).await?,
            None => {
                let block_id = transaction.get_checkpoint_id_filter()?;
                get_transactions_by_checkpoint_id(block_id, &provider).await?
            }
        };

        let result_futures = rpc_transactions
            .iter()
            .map(|t| pick_transaction_fields(t, transaction.fields(), &provider, chain));
        let tx_res = try_join_all(result_futures).await?;

        // Filter and collect results for this chain
        let filtered_tx_res: Vec<TransactionQueryRes> = tx_res
            .into_iter()
            .filter(|t| t.has_value() && transaction.filter(t))
            .collect();

        all_results.extend(filtered_tx_res);
    }

    Ok(all_results)
}

async fn get_transactions_by_ids(
    ids: &Vec<TransactionDigest>,
    provider: &SuiClient,
) -> Result<Vec<RpcTransaction>> {
    let mut tx_futures = Vec::new();
    for id in ids {
        let provider = provider.clone();
        let transation_options = SuiTransactionBlockResponseOptions::new()
            .with_effects()
            .with_events();
        let tx_future = async move {
            provider
                .read_api()
                .get_transaction_with_options(*id, transation_options)
                .await
        };
        tx_futures.push(tx_future);
    }

    let tx_res = try_join_all(tx_futures).await?;

    Ok(tx_res.into_iter().filter_map(|t| Some(t)).collect())
}

async fn get_transactions_by_checkpoint_id(
    checkpoint_id: &CheckpointId,
    provider: &SuiClient,
) -> Result<Vec<RpcTransaction>> {
    match checkpoint_id {
        CheckpointId::Number(n) => {
            let checkpoint = get_checkpoint(n.clone(), provider).await?;
            let option_transaction = SuiTransactionBlockResponseOptions::new()
                .with_effects()
                .with_events();
            let digests = checkpoint.transactions;
            let tnx = provider
                .read_api()
                .multi_get_transactions_with_options(digests, option_transaction)
                .await?;
            Ok(tnx)
        }
        CheckpointId::Range(r) => {
            let checkpoint_numbers = r.resolve_checkpoint_numbers(provider).await?;
            let checkpoints = batch_get_checkpoints(checkpoint_numbers, provider).await?;
            let option_transaction = SuiTransactionBlockResponseOptions::new()
                .with_effects()
                .with_events();
            let mut all_digests = Vec::new();
            for checkpoint in checkpoints {
                all_digests.extend(checkpoint.transactions);
            }

            let txs = provider
                .read_api()
                .multi_get_transactions_with_options(all_digests, option_transaction)
                .await?;

            Ok(txs)
        }
    }
}

async fn pick_transaction_fields(
    tx: &RpcTransaction,
    fields: &Vec<TransactionField>,
    provider: &SuiClient,
    chain: &ChainOrRpc,
) -> Result<TransactionQueryRes> {
    let mut result = TransactionQueryRes::default();
    let chain = chain.to_chain().await?;
    let txn_data = tx.transaction.as_ref().map(|t| &t.data);
    let sender = txn_data.map(|d| d.sender());
    let gas = txn_data.map(|d| d.gas_data());
    let total_events = tx.events.as_ref().map_or(0, |e| e.data.len());
    let kind = tx.transaction.clone().unwrap().data.transaction().name();
    let executed_epoch = tx.effects.clone().unwrap().executed_epoch();

    for field in fields {
        match field {
            TransactionField::Type => {
                result.r#kind = Some(kind.to_string());
            }
            TransactionField::Digest => {
                result.digest = Some(tx.digest.clone());
            }
            TransactionField::GasPrice => {
                result.gas_price = gas.map(|g| g.price);
            }
            TransactionField::Status => {
                result.status = tx.confirmed_local_execution;
            }
            TransactionField::Chain => {
                result.chain = Some(chain.clone());
            }
            TransactionField::Sender => {
                result.sender = sender.copied();
            }
            TransactionField::GasBudget => {
                result.gas_budget = gas.map(|g| g.budget);
            }
            TransactionField::GasUsed => {
                result.gas_used = Some(0);
            }
            TransactionField::ExecutedEpoch => {
                result.executed_epoch = Some(executed_epoch);
            }
            TransactionField::Checkpoint => {
                result.checkpoint = tx.checkpoint;
            }
            TransactionField::TimestampMs => {
                result.timestamp_ms = tx.timestamp_ms;
            }
            TransactionField::TotalEvents => {
                result.total_events = Some(total_events);
            }
            // Implement the rest or use `todo!()`:
            _ => todo!("Field {:?} not yet implemented", field),
        }
    }

    Ok(result)
}
