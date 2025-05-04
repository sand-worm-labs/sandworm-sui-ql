use crate::common::{
    account::{Account, AccountField},
    chain::ChainOrRpc,
    coin::CoinField,
    name_services::NameOrAddress,
    query_result::{AccountQueryRes, CoinQueryRes},
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
pub async fn resolve_coin_query(
    account: &Account,
    chains: &[ChainOrRpc],
) -> Result<Vec<AccountQueryRes>> {
    let mut all_account_futures = Vec::new();

    for chain in chains {
        let provider = SuiClientBuilder::default().build(chain.rpc_url()?).await?;

        // TODO: Handle filter
        // TODO: Remove unwrap
        for account_id in account.ids().unwrap() {
            let fields = account.fields().clone();
            let provider = provider.clone();

            let account_future = async move {
                match account_id {
                    NameOrAddress::Address(address) => {
                        get_account(address, fields, &provider, chain).await
                    }
                    NameOrAddress::Name(name) => {
                        let address = to_address(name).await?;
                        get_account(&address, fields, &provider, chain).await
                    }
                }
            };

            all_account_futures.push(account_future);
        }
    }

    let account_res = try_join_all(all_account_futures).await?;
    Ok(account_res)
}

async fn get_coin(
    address: &SuiAddress,
    fields: Vec<CoinField>,
    provider: &SuiClient,
    chain: &ChainOrRpc,
) -> Result<CoinQueryRes> {
    let mut coin = CoinQueryRes::default();
    let chain = chain.to_chain().await?;

    for field in &fields {
        match field {
            CoinField::Decimals => todo!(),
            CoinField::Name => todo!(),
            CoinField::Symbol => todo!(),
            CoinField::Description => todo!(),
            CoinField::IconUrl => todo!(),
            CoinField::Chain => todo!(),
        }
    }

    Ok(coin)
}
