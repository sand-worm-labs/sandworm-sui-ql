use super::config::Config;
use alloy::transports::http::reqwest::Url;
use anyhow::Result;
use core::fmt;
use eql_macros::EnumVariants;
use pest::iterators::Pairs;
use serde::{Deserialize, Serialize};
use sui_sdk::SuiClientBuilder;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChainOrRpc {
    Chain(Chain),
    Rpc(Url),
}

impl ChainOrRpc {
    pub fn rpc_url(&self) -> Result<Url> {
        match self {
            ChainOrRpc::Chain(chain) => Ok(chain.rpc_url()?.clone()),
            ChainOrRpc::Rpc(url) => Ok(url.clone()),
        }
    }

    pub async fn to_chain(&self) -> Result<Chain> {
        match self {
            ChainOrRpc::Chain(chain) => Ok(chain.clone()),
            ChainOrRpc::Rpc(rpc) => {
                let provider = SuiClientBuilder::default().build(rpc.clone()).await?;
                let chain_id = provider.read_api().get_chain_identifier().await?;
                Err(anyhow::anyhow!("Unknown chain ID: {}", chain_id)) // You only support 3 chains. No detection.
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, EnumVariants, Serialize, Deserialize)]
pub enum Chain {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(thiserror::Error, Debug)]
pub enum ChainError {
    #[error("Invalid chain {0}")]
    InvalidChain(String),
}

impl Chain {
    pub fn from_selector(selector: &str) -> Result<Vec<ChainOrRpc>, ChainError> {
        if selector == "*" {
            let chains = Chain::all_variants();
            let chains = chains
                .into_iter()
                .map(|chain| ChainOrRpc::Chain(chain.clone()))
                .collect::<Vec<ChainOrRpc>>();
            Ok(chains)
        } else {
            let chains = selector
                .split(',')
                .map(str::trim)
                .map(|s| Chain::try_from(s).map(ChainOrRpc::Chain))
                .collect::<Result<Vec<ChainOrRpc>, ChainError>>()?;

            Ok(chains)
        }
    }

    pub fn rpc_url(&self) -> Result<Url> {
        match Config::new().get_chain_default_rpc(self) {
            Ok(Some(url)) => Ok(url),
            Ok(None) => Ok(self.rpc_fallback().parse()?),
            Err(e) => Err(e),
        }
    }

    fn rpc_fallback(&self) -> &str {
        match self {
            Chain::Mainnet => "https://fullnode.mainnet.sui.io:443",
            Chain::Testnet => "https://fullnode.testnet.sui.io:443",
            Chain::Devnet => "https://fullnode.devnet.sui.io:443",
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::Mainnet
    }
}

impl TryFrom<&str> for Chain {
    type Error = ChainError;

    fn try_from(chain: &str) -> Result<Self, Self::Error> {
        match chain {
            "mainnet" => Ok(Chain::Mainnet),
            "testnet" => Ok(Chain::Testnet),
            "devnet" => Ok(Chain::Devnet),
            _ => Err(ChainError::InvalidChain(chain.to_string())),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chain_str = match self {
            Chain::Mainnet => "mainnet",
            Chain::Testnet => "testnet",
            Chain::Devnet => "devnet",
        };
        write!(f, "{}", chain_str)
    }
}
