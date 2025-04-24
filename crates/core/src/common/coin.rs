use super::ens::NameOrAddress;
use crate::interpreter::frontend::parser::Rule;
use alloy::hex::FromHexError;
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::{fmt::{write, Display}, str::FromStr};

#[derive(thiserror::Error, Debug)]
pub enum CoinError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error(transparent)]
    CoinFieldError(#[from] CoinFieldError),

    #[error(transparent)]
    CoinFilterError(#[from] CoinFilterError),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coin {
    id: Option<Vec<NameOrAddress>>,
    filter: Option<Vec<CoinFilter>>,
    fields: Vec<CoinField>,
}

impl Coin {
    pub fn new(
        id: Option<Vec<NameOrAddress>>,
        filter: Option<Vec<CoinFilter>>,
        fields: Vec<CoinField>,
    ) -> Self {
        Self { id, filter, fields }
    }

    pub fn ids(&self) -> Option<&Vec<NameOrAddress>> {
        self.id.as_ref()
    }

    pub fn filter(&self) -> Option<Vec<CoinFilter>> {
        self.filter.clone()
    }

    pub fn fields(&self) -> Vec<CoinField> {
        self.fields.clone()
    }
}

impl TryFrom<Pairs<'_, Rule>> for Coin {
    type Error = CoinError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut fields: Vec<CoinField> = vec![];
        let mut id: Option<Vec<NameOrAddress>> = None;
        let mut filter: Option<Vec<CoinFilter>> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::Coin_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = CoinField::all_variants().to_vec();
                            continue;
                        }
                    }

                    fields = inner_pairs
                        .map(|pair| CoinField::try_from(pair))
                        .collect::<Result<Vec<CoinField>, CoinFieldError>>()?;
                }
                Rule::Coin_id => {
                    if let Some(id) = id.as_mut() {
                        id.push(NameOrAddress::from_str(pair.as_str())?);
                    } else {
                        id = Some(vec![NameOrAddress::from_str(pair.as_str())?]);
                    }
                }
                Rule::Coin_filter_list => {
                    filter = Some(
                        pair.into_inner()
                            .map(|pair| CoinFilter::try_from(pair))
                            .collect::<Result<Vec<CoinFilter>, CoinFilterError>>()?,
                    );
                }
                _ => {
                    return Err(CoinError::UnexpectedToken(pair.as_str().to_string()));
                }
            }
        }

        Ok(Coin { id, filter, fields })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CoinFilterError {
    #[error("Unexpected token {0} for Coin filter")]
    UnexpectedToken(String),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CoinFilter {
    Address(NameOrAddress),
}

// impl TryFrom<Pair<'_, Rule>> for CoinFilter {
//     type Error = CoinFilterError;

//     fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
//         match pair.as_rule() {
//             Rule::address_filter => {
//                 let address = NameOrAddress::from_str(pair.as_str())?;
//                 Ok(CoinFilter::Address(address))
//             }
//             _ => {
//                 return Err(CoinFilterError::UnexpectedToken(
//                     pair.as_str().to_string(),
//                 ));
//             }
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum CoinField {
    Decimals,
    Name,
    Symbol,
    Description,
    IconUrl,
    CoinType,
    TotalSupply,
    Chain,
}

impl Display for CoinField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinField::Name => write!(f, "name"),
            CoinField::Symbol => write!(f, "symbol"),
            CoinField::Description => write!(f, "description"),
            CoinField::IconUrl => write!(f, "icon_url"),
            CoinField::Chain => write!(f, "chain"),
            CoinField::Decimals => write!(f, "decimals"),
            CoinField::CoinType => write!(f, "coin_type"),
            CoinField::TotalSupply => write!(f, "total_supply"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CoinFieldError {
    #[error("Invalid field for entity Coin: {0}")]
    InvalidField(String),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),
}

impl<'a> TryFrom<Pair<'a, Rule>> for CoinField {
    type Error = CoinFieldError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        CoinField::try_from(pair.as_str())
    }
}

impl TryFrom<&str> for CoinField {
    type Error = CoinFieldError;

    fn try_from(value: &str) -> Result<Self, CoinFieldError> {
        match value {
            "name" => Ok(CoinField::Name),
            "symbol" => Ok(CoinField::Symbol),
            "description" => Ok(CoinField::Description),
            "icon_url" => Ok(CoinField::IconUrl),
            "chain" => Ok(CoinField::Chain),
            "decimals" => Ok(CoinField::Decimals),
            "coin_type" => Ok(CoinField::CoinType),
            "total_supply" => Ok(CoinField::TotalSupply),
            _ => Err(CoinFieldError::InvalidField(value.to_string())),
        }
    }
}