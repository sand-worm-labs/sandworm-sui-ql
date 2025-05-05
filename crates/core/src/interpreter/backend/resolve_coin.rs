use crate::common::{
    chain::ChainOrRpc,
    coin::{Coin, CoinField},
    query_result::CoinQueryRes,
};
use anyhow::Result;
use futures::future::try_join_all;
use sui_sdk::{SuiClient, SuiClientBuilder};

pub async fn resolve_coin_query(coin: &Coin, chains: &[ChainOrRpc]) -> Result<Vec<CoinQueryRes>> {
    let mut all_coins_futures = Vec::new();

    for chain in chains {
        let provider = SuiClientBuilder::default().build(chain.rpc_url()?).await?;

        // TODO: Handle filter
        // TODO: Remove unwrap
        for coin_id in coin.ids().unwrap() {
            let fields = coin.fields().clone();
            let provider = provider.clone();

            let coin_future = async move { get_coin(coin_id, fields, &provider, chain).await };

            all_coins_futures.push(coin_future);
        }
    }

    let coin_res = try_join_all(all_coins_futures).await?;
    Ok(coin_res)
}

async fn get_coin(
    coin_id: &String,
    fields: Vec<CoinField>,
    provider: &SuiClient,
    chain: &ChainOrRpc,
) -> Result<CoinQueryRes> {
    let mut coin = CoinQueryRes::default();
    let chain = chain.to_chain().await?;
    let coin_result = provider
        .coin_read_api()
        .get_coin_metadata(coin_id.clone())
        .await?
        .unwrap();

    for field in &fields {
        match field {
            CoinField::Decimals => {
                coin.decimals = Some(coin_result.decimals);
            }
            CoinField::Name => {
                coin.name = Some(coin_result.name.clone());
            }
            CoinField::Symbol => {
                coin.symbol = Some(coin_result.symbol.clone());
            }
            CoinField::Description => {
                coin.description = Some(coin_result.description.clone());
            }
            CoinField::IconUrl => {
                coin.icon_url = coin_result.icon_url.clone();
            }
            CoinField::Chain => {
                coin.chain = Some(chain.clone());
            }
        }
    }

    Ok(coin)
}
