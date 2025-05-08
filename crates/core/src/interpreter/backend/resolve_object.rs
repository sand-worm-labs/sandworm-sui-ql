use crate::common::{
    chain::ChainOrRpc,
    object::{Object, ObjectField},
    query_result::ObjectQueryRes,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sui_json_rpc_types::SuiObjectDataOptions;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::ObjectID;

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
    let mut all_results = Vec::new();
    for chain in chains {
        let provider = SuiClientBuilder::default().build(chain.rpc_url()?).await?;
        let objects_ids = object.ids().unwrap();
        let fields = object.fields().clone();
        let results = get_objects(objects_ids.to_vec(), fields, &provider, chain).await?;
        all_results.extend(results);
    }

    Ok(all_results)
}

async fn get_objects(
    objects_ids: Vec<ObjectID>,
    fields: Vec<ObjectField>,
    provider: &SuiClient,
    chain: &ChainOrRpc,
) -> Result<Vec<ObjectQueryRes>> {
    let mut objects = Vec::new();
    let chain = chain.to_chain().await?;
    let options = SuiObjectDataOptions::default()
        .with_owner()
        .with_previous_transaction()
        .with_bcs()
        .with_type();
    let results = provider
        .read_api()
        .multi_get_object_with_options(objects_ids, options)
        .await?;
    for object in results.iter() {
        let Some(object_data) = object.data.as_ref() else {
            continue;
        };
        let Some(owner_ref) = object_data.owner.as_ref() else {
            continue;
        };
        let Ok(owner) = owner_ref.get_address_owner_address() else {
            continue;
        };

        let mut object_res = ObjectQueryRes::default();
        for field in &fields {
            match field {
                ObjectField::ObjectId => {
                    object_res.object_id = Some(object_data.object_id.to_string());
                }
                ObjectField::Digest => {
                    object_res.digest = Some(object_data.digest.to_string());
                }
                ObjectField::Chain => {
                    object_res.chain = Some(chain.clone());
                }
                ObjectField::Owner => {
                    object_res.owner = Some(owner);
                }
                ObjectField::PreviousTransaction => {
                    object_res.previous_transaction = object_data.previous_transaction;
                }
                ObjectField::StorageRebate => {
                    object_res.storage_rebate = object_data.storage_rebate;
                }
                ObjectField::Version => object_res.version = Some(object_data.version),
            }
        }
        objects.push(object_res);
    }
    Ok(objects)
}
