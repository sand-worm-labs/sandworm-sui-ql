/// Based on foundry-common implementation
/// https://github.com/foundry-rs/foundry/blob/master/crates/common/src/ens.rs
// use self::EnsResolver::EnsResolverInstance;
use alloy::primitives::{address, Address, Keccak256, B256};
use alloy::providers::RootProvider;
use alloy::sol;
use alloy::transports::http::{Client, Http};
use std::fmt::Display;
use std::{borrow::Cow, str::FromStr};
use sui_types::base_types::SuiAddress;

/// Error type for ENS resolution.
#[derive(Debug, thiserror::Error)]
pub enum NSError {
    /// Failed to get resolver from the ENS registry.
    #[error("Failed to get resolver from the ENS registry: {0}")]
    Resolver(alloy::contract::Error),
    /// Failed to get resolver from the ENS registry.
    #[error("ENS resolver not found for name {0:?}")]
    ResolverNotFound(String),
    /// Failed to lookup ENS name from an address.
    #[error("Failed to lookup ENS name from an address: {0}")]
    Lookup(alloy::contract::Error),
    /// Failed to resolve ENS name to an address.
    #[error("Failed to resolve ENS name to an address: {0}")]
    Resolve(alloy::contract::Error),
}

/// ENS name or Ethereum Address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrAddress {
    /// An Name Service (format does not get checked)
    Name(String),
    /// An Sui Address
    Address(SuiAddress),
}

// impl NameOrAddress {
//     /// Resolves the name to an Ethereum Address.
//     pub async fn resolve(
//         &self,
//         provider: &RootProvider<Http<Client>>,
//     ) -> Result<Address, EnsError> {
//         match self {
//             Self::Name(name) => self.resolve_name(name, provider).await,
//             Self::Address(addr) => Ok(*addr),
//         }
//     }

//     async fn resolve_name(
//         &self,
//         name: &str,
//         provider: &RootProvider<Http<Client>>,
//     ) -> Result<Address, EnsError> {
//         let node = namehash(name);
//         let registry = EnsRegistry::new(ENS_ADDRESS, provider.clone());

//         let address = registry
//             .resolver(node)
//             .call()
//             .await
//             .map_err(EnsError::Resolver)?
//             ._0;
//         if address == Address::ZERO {
//             return Err(EnsError::ResolverNotFound(String::from(
//                 "Resolved to zero address",
//             )));
//         }

//         let resolver = EnsResolverInstance::new(address, provider);
//         let addr = resolver
//             .addr(node)
//             .call()
//             .await
//             .map_err(EnsError::Resolve)
//             .inspect_err(|e| eprintln!("{e:?}"))?
//             ._0;

//         Ok(addr)
//     }
// }

impl From<String> for NameOrAddress {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}

impl From<&String> for NameOrAddress {
    fn from(name: &String) -> Self {
        Self::Name(name.clone())
    }
}

impl From<SuiAddress> for NameOrAddress {
    fn from(addr: SuiAddress) -> Self {
        Self::Address(addr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy::primitives::hex;

    // fn assert_hex(hash: B256, val: &str) {
    //     assert_eq!(hash.0[..], hex::decode(val).unwrap()[..]);
    // }

    #[test]
    fn test_namehash() {
        // for (name, expected) in &[
        //     (
        //         "",
        //         "0x0000000000000000000000000000000000000000000000000000000000000000",
        //     ),
        //     (
        //         "eth",
        //         "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae",
        //     ),
        //     (
        //         "foo.eth",
        //         "0xde9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f",
        //     ),
        //     (
        //         "alice.eth",
        //         "0x787192fc5378cc32aa956ddfdedbf26b24e8d78e40109add0eea2c1a012c3dec",
        //     ),
        //     (
        //         "ret↩️rn.eth",
        //         "0x3de5f4c02db61b221e7de7f1c40e29b6e2f07eb48d65bf7e304715cd9ed33b24",
        //     ),
        // ] {
        //     assert_hex(namehash(name), expected);
        // }
    }

    #[test]
    fn test_reverse_address() {
        // for (addr, expected) in [
        //     (
        //         "0x314159265dd8dbb310642f98f50c066173c1259b",
        //         "314159265dd8dbb310642f98f50c066173c1259b.addr.reverse",
        //     ),
        //     (
        //         "0x28679A1a632125fbBf7A68d850E50623194A709E",
        //         "28679a1a632125fbbf7a68d850e50623194a709e.addr.reverse",
        //     ),
        // ] {
        //     assert_eq!(reverse_address(&addr.parse().unwrap()), expected, "{addr}");
        // }
    }
}
