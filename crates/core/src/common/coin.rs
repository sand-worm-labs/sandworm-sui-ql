use crate::interpreter::frontend::parser::Rule;
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    str::FromStr,
};

#[derive(thiserror::Error, Debug)]
pub enum CoinError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error(transparent)]
    CoinFieldError(#[from] CoinFieldError),

    #[error(transparent)]
    CoinFilterError(#[from] CoinFilterError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coin {
    id: Option<Vec<String>>,
    filter: Option<Vec<CoinFilter>>,
    fields: Vec<CoinField>,
}

impl Coin {
    pub fn new(
        id: Option<Vec<String>>,
        filter: Option<Vec<CoinFilter>>,
        fields: Vec<CoinField>,
    ) -> Self {
        Self { id, filter, fields }
    }

    pub fn ids(&self) -> Option<&Vec<String>> {
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
        let mut id: Option<Vec<String>> = None;
        let mut filter: Option<Vec<CoinFilter>> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::coin_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = CoinField::all_variants().to_vec();
                            continue;
                        }
                    }

                    fields = inner_pairs
                        .map(CoinField::try_from)
                        .collect::<Result<Vec<CoinField>, CoinFieldError>>()?;
                }
                Rule::coin_id => {
                    let val = pair.as_str().to_string();
                    if let Some(ref mut vec) = id {
                        vec.push(val);
                    } else {
                        id = Some(vec![val]);
                    }
                }
                Rule::coin_filter_list => {
                    filter = Some(
                        pair.into_inner()
                            .map(CoinFilter::try_from)
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

    #[error("Failed to parse coin id: {0}")]
    CoinParseError(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CoinFilter {
    CoinId(String),
}

impl TryFrom<Pair<'_, Rule>> for CoinFilter {
    type Error = CoinFilterError;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::move_struct_tag => {
                let id = String::from_str(pair.as_str()).map_err(|e| CoinFilterError::CoinParseError(e.to_string()))?;
                Ok(CoinFilter::CoinId(id))
            }
            _ => Err(CoinFilterError::UnexpectedToken(pair.as_str().to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum CoinField {
    Decimals,
    Name,
    Symbol,
    Description,
    IconUrl,
    Chain,
}

impl Display for CoinField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CoinField::Name => "name",
            CoinField::Symbol => "symbol",
            CoinField::Description => "description",
            CoinField::IconUrl => "icon_url",
            CoinField::Chain => "chain",
            CoinField::Decimals => "decimals",
        };
        write!(f, "{}", s)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CoinFieldError {
    #[error("Invalid field for entity Coin: {0}")]
    InvalidField(String),
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
            _ => Err(CoinFieldError::InvalidField(value.to_string())),
        }
    }
}
