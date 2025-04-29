use crate::common::{
    chain::{Chain, ChainOrRpc},
    checkpoint::{
        get_checkpoint_number_from_tag, Checkpoint, CheckpointField, CheckpointId,
        CheckpointNumberOrTag,
    },
    query_result::CheckpointQueryRes,
};
use anyhow::Result;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sui_json_rpc_types::Checkpoint as RpcCheckpoint;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::digests::CheckpointDigest;


#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum CheckpointResolverErrors {
    #[error("Unable to fetch block number for tag {0}")]
    UnableToFetchCheckpointNumber(CheckpointNumberOrTag),
    #[error("Mismatch between Entity and EntityId, {0} can't be resolved as a block id")]
    MismatchEntityAndEntityId(String),
    #[error("Missing block ids")]
    IdsNotSet,
}

async fn resolve_checkpoint_id(id: &CheckpointId, provider: &SuiClient) -> Result<Vec<u64>> {
    let block_numbers = match id {
        CheckpointId::Range(checkpoint_range) => {
            checkpoint_range
                .resolve_checkpoint_numbers(&provider)
                .await?
        }
        CheckpointId::Number(checkpoint_number) => {
            resolve_cheeckpoint_numbers(&[checkpoint_number.clone()], provider).await?
        }
    };

    Ok(block_numbers)
}

pub async fn resolve_checkpoint_query(
    checkpoints: &Checkpoint,
    chains: &[ChainOrRpc],
) -> Result<Vec<CheckpointQueryRes>> {
    let mut all_chain_futures = Vec::new();

    let ids = match checkpoints.ids() {
        Some(ids) => ids,
        None => return Err(CheckpointResolverErrors::IdsNotSet.into()),
    };

    for chain in chains {
        let fields = checkpoints.fields().clone();

        let chain_future = async move {
            let provider =  SuiClientBuilder::default().build(chain.rpc_url()?).await?;
            let chain = chain.to_chain().await?;
            let mut all_checkpoint_futures = Vec::new();

            for id in ids {
                let chain_clone = chain.clone();
                let provider_clone = provider.clone();
                let fields = fields.clone();

                let checkpoint_id = resolve_checkpoint_id(&id, &provider_clone).await?;
                let checkpoint_future = async move {
                    get_filtered_checkpoints(checkpoint_id,fields, &provider_clone, &chain_clone).await
                };
                all_checkpoint_futures.push(checkpoint_future);
            }

            let chain_blocks = try_join_all(all_checkpoint_futures).await?;
            Ok::<Vec<CheckpointQueryRes>, anyhow::Error>(chain_blocks.concat())
        };

        all_chain_futures.push(chain_future);
    }

    let all_chain_blocks = try_join_all(all_chain_futures).await?;
    Ok(all_chain_blocks.concat())
}

async fn get_filtered_checkpoints(
    checkpoint_numbers: Vec<u64>,
    fields: Vec<CheckpointField>,
    provider: &SuiClient,
    chain: &Chain,
) -> Result<Vec<CheckpointQueryRes>> {
    let checkpoint = batch_get_checkpoints(checkpoint_numbers, provider).await?;
    Ok(checkpoint
        .into_iter()
        .map(|checkpoint| filter_fields(checkpoint, &fields, &chain))
        .collect())
}

// TODO: this method only exists here because it wasn't implemented on the BlockId struct yet.
// BlockRange has a similar implementation and should be unified.
async fn resolve_cheeckpoint_numbers(
    checkpoint_numbers: &[CheckpointNumberOrTag],
    provider: &SuiClient,
) -> Result<Vec<u64>> {
    let mut checkpoint_number_futures = Vec::new();

    for checkpoint_number in checkpoint_numbers {
        let block_number_future =
            async move { get_checkpoint_number_from_tag(provider, checkpoint_number).await };
        checkpoint_number_futures.push(block_number_future);
    }

    let chepoint_numbers = try_join_all(checkpoint_number_futures).await?;
    Ok(chepoint_numbers)
}

pub async fn batch_get_checkpoints(
    checkpoint_numbers: Vec<u64>,
    provider: &SuiClient,
) -> Result<Vec<RpcCheckpoint>> {
    let mut checkpoin_futures = Vec::new();

    for checkpoint_number in checkpoint_numbers {
        let checkpoint_future = async move {
            get_checkpoint(
                CheckpointNumberOrTag::Number(checkpoint_number),
                provider
            )
            .await
        };
        checkpoin_futures.push(checkpoint_future);
    }

    let checkpoint_results = try_join_all(checkpoin_futures).await?;
    Ok(checkpoint_results)
}

pub async fn get_checkpoint(
    checkpoint_id: CheckpointNumberOrTag,
    provider: &SuiClient,
) -> Result<RpcCheckpoint> {
    let sui_checkpoint_id = checkpoint_id.to_sui_checkpoint_id().ok_or(CheckpointResolverErrors::UnableToFetchCheckpointNumber(checkpoint_id.clone()))?;
    let checkpoint = provider.read_api().get_checkpoint(sui_checkpoint_id).await.map_err(|_|{
        CheckpointResolverErrors::UnableToFetchCheckpointNumber(checkpoint_id.clone())
    })?;
    Ok(checkpoint) 
}

fn filter_fields(
    checkpoint: RpcCheckpoint,
    fields: &[CheckpointField],
    chain: &Chain,
) -> CheckpointQueryRes {
    let mut result = CheckpointQueryRes::default();

    for field in fields {
        match field {
            CheckpointField::Timestamp => {
                                result.timestamp = Some(checkpoint.timestamp_ms);
                            }
            CheckpointField::Number => {
                                result.number = Some(checkpoint.sequence_number);
                            }
            CheckpointField::Transactions => {
                                result.transactions = Some(checkpoint.transactions.len());
                            }
            CheckpointField::Epoch => {
                                result.epoch = Some(checkpoint.epoch);
                            }
            CheckpointField::Digest => {
                                result.digest = Some(checkpoint.digest.base58_encode());
                            }
            CheckpointField::ComputationCost => {
                                result.computation_cost = Some(checkpoint.epoch_rolling_gas_cost_summary.storage_cost);
                            }
            CheckpointField::StorageCost => {
                            result.storage_cost = Some(checkpoint.epoch_rolling_gas_cost_summary.storage_cost);
            },
            CheckpointField::StorageRebate => {
                            result.storage_rebate = Some(checkpoint.epoch_rolling_gas_cost_summary.storage_rebate);
            },
            CheckpointField::NonRefundableStorageFee => {
                result.non_refundable_storage_fee = Some(checkpoint.epoch_rolling_gas_cost_summary.non_refundable_storage_fee);
            },
            CheckpointField::PreviousDigest => {
                                let previous_digest = checkpoint.previous_digest.unwrap_or(CheckpointDigest::default()).base58_encode();
                                result.previous_digest = Some(previous_digest);
                            }
            CheckpointField::ValidatorSignature => {
                                result.validator_signature = Some(checkpoint.validator_signature.to_string());
                            }
            CheckpointField::Chain => {
                                result.chain = Some(chain.clone());
                            }
            CheckpointField::NetworkTotalTransactions => {
                                result.network_total_transactions = Some(checkpoint.network_total_transactions);
                    },
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{checkpoint::CheckpointRange, chain::Chain};

    #[tokio::test]
    async fn test_error_when_start_block_is_greater_than_end_block() {
        let start_block = 10;
        let end_block = 5;
        // Empty fields for simplicity
        let fields = vec![];
        let chain = ChainOrRpc::Chain(Chain::Mainnet);
        let block = Checkpoint::new(
            Some(vec![CheckpointId::Range(CheckpointRange::new(
                start_block.into(),
                Some(end_block.into()),
            ))]),
            None,
            fields,
        );

        let result = resolve_checkpoint_query(&block, &[chain]).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Start checkpoint must be less than end checkpoint"
        );
    }
}
