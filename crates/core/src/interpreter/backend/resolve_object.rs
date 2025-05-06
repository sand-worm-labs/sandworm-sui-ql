use crate::common::{
    account::{Account, AccountField}, chain::ChainOrRpc, coin::CoinField, name_services::NameOrAddress, object::Object, query_result::{CoinQueryRes, ObjectQueryRes}
};
use anyhow::Result;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sui_json_rpc_types::StakeStatus;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::SuiAddress;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum AccountResolverErrors {
    #[error("Mismatch between Entity and EntityId, {0} can't be resolved as a account id")]
    MismatchEntityAndEntityId(String),
}

/// Resolve the query to get accounts after receiving an account entity expression
/// Iterate through entity_ids and map them to a futures list. Execute all futures concurrently and collect the results.
pub async fn resolve_object_query(
    object: &Object,
    chains: &[ChainOrRpc],
) -> Result<Vec<ObjectQueryRes>> {
    Ok([].into())
}
