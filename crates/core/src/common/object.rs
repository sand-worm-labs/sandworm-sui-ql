// use super::ens::NameOrAddress;
use crate::interpreter::frontend::parser::Rule;
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use sui_types::base_types::{ObjectID, ObjectIDParseError};

#[derive(thiserror::Error, Debug)]
pub enum ObjectError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error(transparent)]
    ObjectFieldError(#[from] ObjectFieldError),

    #[error(transparent)]
    ObjectFilterError(#[from] ObjectFilterError),

    #[error(transparent)]
    ObjectParseError(#[from] ObjectIDParseError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Object {
    id: Option<Vec<ObjectID>>,
    filter: Option<Vec<ObjectFilter>>,
    fields: Vec<ObjectField>,
}

impl Object {
    pub fn new(
        id: Option<Vec<ObjectID>>,
        filter: Option<Vec<ObjectFilter>>,
        fields: Vec<ObjectField>,
    ) -> Self {
        Self { id, filter, fields }
    }

    pub fn ids(&self) -> Option<&Vec<ObjectID>> {
        self.id.as_ref()
    }

    pub fn filter(&self) -> Option<Vec<ObjectFilter>> {
        self.filter.clone()
    }

    pub fn fields(&self) -> Vec<ObjectField> {
        self.fields.clone()
    }
}

impl TryFrom<Pairs<'_, Rule>> for Object {
    type Error = ObjectError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut fields: Vec<ObjectField> = vec![];
        let mut id: Option<Vec<ObjectID>> = None;
        let mut filter: Option<Vec<ObjectFilter>> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::object_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = ObjectField::all_variants().to_vec();
                            continue;
                        }
                    }

                    fields = inner_pairs
                        .map(|pair| ObjectField::try_from(pair))
                        .collect::<Result<Vec<ObjectField>, ObjectFieldError>>()?;
                }
                Rule::object_id => {
                    if let Some(id) = id.as_mut() {
                        id.push(ObjectID::from_str(pair.as_str())?);
                    } else {
                        id = Some(vec![ObjectID::from_str(pair.as_str())?]);
                    }
                }
                Rule::object_filter_list => {
                    filter = Some(
                        pair.into_inner()
                            .map(|pair| ObjectFilter::try_from(pair))
                            .collect::<Result<Vec<ObjectFilter>, ObjectFilterError>>()?,
                    );
                }
                _ => {
                    return Err(ObjectError::UnexpectedToken(pair.as_str().to_string()));
                }
            }
        }

        Ok(Object { id, filter, fields })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ObjectFilterError {
    #[error("Unexpected token {0} for Object filter")]
    UnexpectedToken(String),

    #[error(transparent)]
    ObjectParseError(#[from] ObjectIDParseError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ObjectFilter {
    ObjectId(ObjectID),
}

impl TryFrom<Pair<'_, Rule>> for ObjectFilter {
    type Error = ObjectFilterError;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::object_filter_template => {
                let address = ObjectID::from_str(pair.as_str())?;
                Ok(ObjectFilter::ObjectId(address))
            }
            _ => {
                return Err(ObjectFilterError::UnexpectedToken(
                    pair.as_str().to_string(),
                ));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum ObjectField {
    ObjectId,
    Version,
    Digest,
    Owner,
    PreviousTransaction,
    StorageRebate,
    Chain,
}

impl std::fmt::Display for ObjectField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ObjectField::ObjectId => "object_id",
            ObjectField::Version => "version",
            ObjectField::Digest => "digest",
            ObjectField::Owner => "owner",
            ObjectField::PreviousTransaction => "previous_transaction",
            ObjectField::StorageRebate => "storage_rebate",
            ObjectField::Chain => "chain",
        };
        write!(f, "{s}")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ObjectFieldError {
    #[error("Invalid field for entity Object: {0}")]
    InvalidField(String),
}

impl<'a> TryFrom<Pair<'a, Rule>> for ObjectField {
    type Error = ObjectFieldError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        ObjectField::try_from(pair.as_str())
    }
}

impl TryFrom<&str> for ObjectField {
    type Error = ObjectFieldError;

    fn try_from(value: &str) -> Result<Self, ObjectFieldError> {
        match value {
            "object_id" => Ok(ObjectField::ObjectId),
            "version" => Ok(ObjectField::Version),
            "digest" => Ok(ObjectField::Digest),
            "owner" => Ok(ObjectField::Owner),
            "previous_transaction" => Ok(ObjectField::PreviousTransaction),
            "storage_rebate" => Ok(ObjectField::StorageRebate),
            _ => Err(ObjectFieldError::InvalidField(value.to_string())),
        }
    }
}
