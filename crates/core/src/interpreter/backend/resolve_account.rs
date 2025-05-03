use crate::common::{
    account::{Account, AccountField},
    chain::ChainOrRpc,
    name_services::NameOrAddress,
    query_result::AccountQueryRes,
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
pub async fn resolve_account_query(
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

async fn get_account(
    address: &SuiAddress,
    fields: Vec<AccountField>,
    provider: &SuiClient,
    chain: &ChainOrRpc,
) -> Result<AccountQueryRes> {
    let mut account = AccountQueryRes::default();
    let chain = chain.to_chain().await?;
    let stakes = provider
        .governance_api()
        .get_stakes(*address)
        .await
        .unwrap()
        .into_iter()
        .flat_map(|v| v.stakes);
    let active_delegations = stakes
        .clone()
        .filter(|s| matches!(s.status, StakeStatus::Active { .. }))
        .count();
    let t: u128 = stakes.map(|s| s.principal as u128).sum();
    let coins = provider.coin_read_api().get_all_balances(*address).await.unwrap().len();

    for field in &fields {
        match field {
            AccountField::SuiBalance => {
                if let Ok(balance) = provider.coin_read_api().get_balance(*address, None).await {
                    account.sui_balance = Some(balance.total_balance);
                } else {
                    account.sui_balance = None;
                }
            }
            AccountField::Address => {
                account.address = Some(*address);
            }
            AccountField::Chain => {
                account.chain = Some(chain.clone());
            }
            AccountField::CoinOwned => {
                account.coin_owned = Some(coins);
            }
            AccountField::StakedAmount => {
                account.staked_amount = Some(t);
            }
            AccountField::ActiveDelegations => {
                account.active_delegations = Some(active_delegations);
            }
        }
    }

    Ok(account)
}

async fn to_address(name: &String) -> Result<SuiAddress> {
    let provider = SuiClientBuilder::default().build_mainnet().await.unwrap();
    let address = NameOrAddress::Name(name.clone()).resolve(&provider).await?;
    Ok(address)
}
