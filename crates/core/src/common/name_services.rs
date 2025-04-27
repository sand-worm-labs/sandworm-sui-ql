use std::fmt::Display;
use std::str::FromStr;
use sui_json_rpc_api::IndexerApiClient;
use sui_sdk::SuiClient;
use sui_types::base_types::SuiAddress;

/// Error type for NS resolution.
#[derive(Debug, thiserror::Error)]
pub enum NSError {
    /// Failed to get resolver from the NS registry.
    #[error("NS resolver not found for name {0:?}")]
    ResolverNotFound(String),
    /// Failed to resolve NS name to an address.
    #[error("Failed to resolve NS name to an address: {0}")]
    Resolve(String),

    #[error("Failed to resolve address to an name: {0}")]
    ResolveAddress(String),
}

/// NS name or Ethereum Address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrAddress {
    /// An Name Service (format does not get checked)
    Name(String),
    /// An Sui Address
    Address(SuiAddress),
}

impl NameOrAddress {
    /// Resolves the name to an Ethereum Address.
    pub async fn resolve(&self, provider: &SuiClient) -> Result<SuiAddress, NSError> {
        match self {
            Self::Name(name) => self.resolve_name(name, provider).await,
            Self::Address(addr) => Ok(*addr),
        }
    }

    pub async fn reverse_lookup(&self, provider: &SuiClient) -> Result<String, NSError> {
        match self {
            Self::Name(name) => Ok(name.clone()),
            Self::Address(addr) => self.resolve_address(*addr, provider).await,
        }
    }

    async fn resolve_name(&self, name: &str, provider: &SuiClient) -> Result<SuiAddress, NSError> {
        let request_client = provider.http();
        let address = request_client
            .resolve_name_service_address(name.to_string())
            .await
            .map_err(|e| NSError::Resolve(e.to_string()))?;
        let address = match address {
            Some(addr) => Ok(addr),
            None => Err(NSError::ResolverNotFound(String::from(
                "Resolved to zero address",
            ))),
        }?;
        if address == SuiAddress::ZERO {
            return Err(NSError::ResolverNotFound(String::from(
                "Resolved to zero address",
            )));
        }
        Ok(address)
    }

    async fn resolve_address(
        &self,
        address: SuiAddress,
        provider: &SuiClient,
    ) -> Result<String, NSError> {
        let request_client = provider.http();
        let data = request_client
            .resolve_name_service_names(address, None, None)
            .await
            .map_err(|e| NSError::Resolve(e.to_string()))?
            .data;
        if data.len() == 0 {
            Err(NSError::ResolveAddress(String::from("No address found")))
        } else {
            Ok(data[0].clone())
        }
    }
}

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

impl FromStr for NameOrAddress {
    type Err = NSError; // <- You forgot this. Idiot mistake, fix it.

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(".sui") {
            return Ok(Self::Name(s.to_string()));
        }

        if s.len() == 42 && s.starts_with("0x") {
            match SuiAddress::from_str(s) {
                Ok(addr) => return Ok(Self::Address(addr)),
                Err(_) => {}
            }
        }
        Err(NSError::ResolveAddress(s.to_string()))
    }
}

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use super::*;
    use sui_sdk::SuiClientBuilder;
    use sui_types::base_types::SuiAddress;

    #[tokio::test]
    async fn test_resolve_address() {
        let provider = SuiClientBuilder::default().build_mainnet().await.unwrap();
        for (name, expected) in [
            (
                "test.sui",
                "0x3e04ea76cee7d2db4f41c2972ac8d929606d89f7293320f0886abb41a578190c",
            ),
            (
                "example.sui",
                "0x214a4199264348df2364acd683a3971a9927a5252747f4e0776f0506922f9db0",
            ),
            (
                "data.sui",
                "0xc862c5a237beaece4fc7f1a36f4e4ba93d78790c12c777bb6268c5c0b5585813",
            ),
        ] {
            let name_or_address = NameOrAddress::Name(name.to_string());
            let resolved = name_or_address.resolve(&provider).await.unwrap();
            assert_eq!(resolved, SuiAddress::from_str(expected).unwrap());
        }
    }

    #[tokio::test]
    async fn test_resolve_name() {
        let provider = SuiClientBuilder::default().build_mainnet().await.unwrap();
        for (name, address) in [
            (
                "test.sui",
                "0x3e04ea76cee7d2db4f41c2972ac8d929606d89f7293320f0886abb41a578190c",
            ),
            (
                "example.sui",
                "0x214a4199264348df2364acd683a3971a9927a5252747f4e0776f0506922f9db0",
            ),
            (
                "data.sui",
                "0xc862c5a237beaece4fc7f1a36f4e4ba93d78790c12c777bb6268c5c0b5585813",
            ),
        ] {
            let address = SuiAddress::from_str(address).unwrap();
            let name_or_address = NameOrAddress::Address(address);
            let resolved = name_or_address.reverse_lookup(&provider).await.unwrap();
            assert_eq!(resolved, name);
        }
    }

    #[tokio::test]
    async fn test_resolve_address_failed() {
        let provider = SuiClientBuilder::default().build_mainnet().await.unwrap();
        for name in [
            "nonexistent1234567890.sui",
            "thisshouldnotexist.sui",
            "fakenamespace.sui",
        ] {
            let name_or_address = NameOrAddress::Name(name.to_string());
            let result = name_or_address.resolve(&provider).await;

            assert!(
                matches!(result, Err(NSError::ResolverNotFound(_))),
                "Expected ResolverNotFound error for name {}, got {:?}",
                name,
                result
            );
        }
    }
}
