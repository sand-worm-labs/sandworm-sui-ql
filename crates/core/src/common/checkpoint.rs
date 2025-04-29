use super::entity_id::{parse_checkpoint_number_or_tag, EntityIdError};
use crate::interpreter::frontend::parser::Rule;
use anyhow::Result;
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use sui_sdk::{rpc_types::CheckpointedObjectID, SuiClient};
use sui_json_rpc_types::{CheckpointId as SuiCheckpointId};

#[derive(thiserror::Error, Debug)]
pub enum CheckpointNumberOrTagError {
    #[error("Invalid checkpoint range: {0}")]
    InvalidCheckpointRange(String),
}

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum CheckpointNumberOrTag {
    Number(u64),
    Latest,
    Earliest,
}

impl CheckpointNumberOrTag {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<u64> {
        match *self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_latest(&self) -> bool {
        matches!(self, Self::Latest)
    }

    pub fn is_earliest(&self) -> bool {
        matches!(self, Self::Earliest)
    }

    pub fn to_sui_checkpoint_id(&self) -> Option<SuiCheckpointId> {
       self.as_number().map(SuiCheckpointId::SequenceNumber)
    }

}

impl From<u64> for CheckpointNumberOrTag {
    fn from(num: u64) -> Self {
        Self::Number(num)
    }
}

impl fmt::Display for CheckpointNumberOrTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointNumberOrTag::Number(n) => write!(f, "{n}"),
            CheckpointNumberOrTag::Latest => write!(f, "latest"),
            CheckpointNumberOrTag::Earliest => write!(f, "earliest"),
        }
    }
}

impl FromStr for CheckpointNumberOrTag {
    type Err = CheckpointNumberOrTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            "earliest" => Ok(Self::Number(0)),
            _ => s
                .parse::<u64>()
                .map(Self::Number)
                .map_err(|_| CheckpointNumberOrTagError::InvalidCheckpointRange(s.to_string())),
        }
    }
}

impl serde::Serialize for CheckpointNumberOrTag{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
          match *self {
            Self::Number(x) => serializer.serialize_str(&format!("0x{x:x}")),
            Self::Latest => serializer.serialize_str("latest"),
            Self::Earliest => serializer.serialize_str("earliest"),

        }
    }
}


impl<'de> serde::Deserialize<'de> for CheckpointNumberOrTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        s.parse().map_err(serde::de::Error::custom)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum CheckpointError {
    #[error("Unexpected token {0} for block")]
    UnexpectedToken(String),

    #[error(transparent)]
    CheckpointFilterError(#[from] CheckpointFilterError),

    #[error(transparent)]
    CheckpointFieldError(#[from] CheckpointFieldError),

    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CheckpointId {
    Number(CheckpointNumberOrTag),
    Range(CheckpointRange),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Checkpoint {
    // TODO: ids should be mandatory
    // TODO: ids should be a HashSet
    ids: Option<Vec<CheckpointId>>,
    filter: Option<Vec<CheckpointFilter>>,
    fields: Vec<CheckpointField>,
}

impl Checkpoint {
    pub fn new(
        ids: Option<Vec<CheckpointId>>,
        filter: Option<Vec<CheckpointFilter>>,
        fields: Vec<CheckpointField>,
    ) -> Self {
        Self {
            ids,
            filter,
            fields,
        }
    }

    pub fn ids(&self) -> Option<&Vec<CheckpointId>> {
        self.ids.as_ref()
    }

    pub fn fields(&self) -> &Vec<CheckpointField> {
        &self.fields
    }

    pub fn filters(&self) -> Option<&Vec<CheckpointFilter>> {
        self.filter.as_ref()
    }
}

impl TryFrom<Pairs<'_, Rule>> for Checkpoint {
    type Error = CheckpointError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut fields: Vec<CheckpointField> = vec![];
        let mut ids: Vec<CheckpointId> = vec![];
        let mut filter: Option<Vec<CheckpointFilter>> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::checkpoint_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = CheckpointField::all_variants().to_vec();
                            continue;
                        }
                    }

                    fields = inner_pairs
                        .map(|pair| CheckpointField::try_from(pair.as_str()))
                        .collect::<Result<Vec<CheckpointField>, CheckpointFieldError>>()?;
                }
                // TODO: handle block number list
                Rule::checkpoint_id => {
                    for inner_pair in pair.into_inner() {
                        match inner_pair.as_rule() {
                            Rule::checkpoint_range => {
                                let checkpoint_id = inner_pair.as_str();
                                let (start, end) = match checkpoint_id.split_once(":") {
                                    Some((start, end)) => {
                                        let start = parse_checkpoint_number_or_tag(start)?;
                                        let end = parse_checkpoint_number_or_tag(end)?;
                                        (start, Some(end))
                                    }
                                    None => parse_checkpoint_number_or_tag(checkpoint_id)
                                        .map(|start| (start, None))?,
                                };
                                ids.push(CheckpointId::Range(CheckpointRange::new(start, end)));
                            }
                            Rule::checkpoint_tag_or_number => {
                                ids.push(CheckpointId::Number(parse_checkpoint_number_or_tag(
                                    inner_pair.as_str(),
                                )?));
                            }
                            _ => {
                                return Err(CheckpointError::UnexpectedToken(
                                    inner_pair.as_str().to_string(),
                                ));
                            }
                        }
                    }
                }
                Rule::checkpoint_filter => {
                    filter = Some(
                        pair.into_inner()
                            .map(|pair| CheckpointFilter::try_from(pair))
                            .collect::<Result<Vec<CheckpointFilter>, CheckpointFilterError>>()?,
                    );
                }
                _ => {
                    return Err(CheckpointError::UnexpectedToken(pair.as_str().to_string()));
                }
            }
        }

        Ok(Checkpoint {
            ids: Some(ids),
            filter,
            fields,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckpointFilterError {
    #[error("Invalid block filter property: {0}")]
    InvalidCheckpointFilterProperty(String),

    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CheckpointFilter {
    Range(CheckpointRange),
}

impl TryFrom<Pair<'_, Rule>> for CheckpointFilter {
    type Error = CheckpointFilterError;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::checkpointrange_filter => {
                let range = value.as_str().trim_start_matches("block ").trim();
                let (start, end) = match range.split_once(":") {
                    //if ":" is present, we have an start and an end.
                    Some((start, end)) => (
                        parse_checkpoint_number_or_tag(start)?,
                        Some(parse_checkpoint_number_or_tag(end)?),
                    ),
                    //else we only have start.
                    None => (parse_checkpoint_number_or_tag(range)?, None),
                };
                Ok(CheckpointFilter::Range(CheckpointRange { start, end }))
            }
            _ => Err(CheckpointFilterError::InvalidCheckpointFilterProperty(
                value.as_str().to_string(),
            )),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CheckpointFieldError {
    #[error("Invalid property for entity block: {0}")]
    InvalidCheckpointField(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum CheckpointField {
    Epoch,
    Number,
    Digest,
    NetworkTotalTransactions,
    PreviousDigest,
    Timestamp,
    Transactions,
    ValidatorSignature,
    Chain,
    ComputationCost,
    StorageCost,
    StorageRebate,
    NonRefundableStorageFee,
}
impl Display for CheckpointField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointField::Epoch => write!(f, "epoch"),
            CheckpointField::Number => write!(f, "number"),
            CheckpointField::Digest => write!(f, "digest"),
            CheckpointField::NetworkTotalTransactions => write!(f, "network_total_transactions"),
            CheckpointField::PreviousDigest => write!(f, "previous_digest"),
            CheckpointField::Timestamp => write!(f, "timestamp"),
            CheckpointField::Transactions => write!(f, "transactions"),
            CheckpointField::ValidatorSignature => write!(f, "validator_signature"),
            CheckpointField::Chain => write!(f, "chain"),
            CheckpointField::ComputationCost => write!(f, "computation_cost"),
            CheckpointField::StorageCost => write!(f, "storage_cost"),
            CheckpointField::StorageRebate => write!(f, "storage_rebate"),
            CheckpointField::NonRefundableStorageFee => write!(f, "non_refundable_storage_fee"),
        }
    }
}

impl TryFrom<&str> for CheckpointField {
    type Error = CheckpointFieldError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "epoch" => Ok(CheckpointField::Epoch),
            "number" => Ok(CheckpointField::Number),
            "digest" => Ok(CheckpointField::Digest),
            "network_total_transactions" => Ok(CheckpointField::NetworkTotalTransactions),
            "previous_digest" => Ok(CheckpointField::PreviousDigest),
            "timestamp" => Ok(CheckpointField::Timestamp),
            "transactions" => Ok(CheckpointField::Transactions),
            "validator_signature" => Ok(CheckpointField::ValidatorSignature),
            "chain" => Ok(CheckpointField::Chain),
            "computation_cost" => Ok(CheckpointField::ComputationCost),
            "storage_cost" => Ok(CheckpointField::StorageCost),
            "storage_rebate" => Ok(CheckpointField::StorageRebate),
            "non_refundable_storage_fee" => Ok(CheckpointField::NonRefundableStorageFee),
            invalid_field => Err(CheckpointFieldError::InvalidCheckpointField(
                invalid_field.to_string(),
            )),
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum CheckpointRangeError {
    #[error("Unable to fetch block number {0}")]
    UnableToFetchCheckpointNumber(CheckpointNumberOrTag),
    #[error("Start block must be less than end block")]
    StartCheckpointMustBeLessThanEndCheckpoint,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CheckpointRange {
    start: CheckpointNumberOrTag,
    end: Option<CheckpointNumberOrTag>,
}

impl CheckpointRange {
    pub fn new(start: CheckpointNumberOrTag, end: Option<CheckpointNumberOrTag>) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> (CheckpointNumberOrTag, Option<CheckpointNumberOrTag>) {
        (self.start, self.end)
    }

    pub fn start(&self) -> CheckpointNumberOrTag {
        self.start
    }

    pub fn end(&self) -> Option<CheckpointNumberOrTag> {
        self.end
    }

    pub async fn resolve_checkpoint_numbers(&self, provider: &SuiClient) -> Result<Vec<u64>> {
        let (start_block, end_block) = self.range();
        let start_block_number = get_checkpoint_number_from_tag(provider, &start_block).await?;

        let end_block_number = match end_block {
            Some(end) => Some(get_checkpoint_number_from_tag(provider, &end).await?),
            None => None,
        };

        if let Some(end) = end_block_number {
            if start_block_number > end {
                return Err(
                    CheckpointRangeError::StartCheckpointMustBeLessThanEndCheckpoint.into(),
                );
            }
        }

        match end_block_number {
            Some(end) => {
                let range = start_block_number..=end;
                return Ok(range.collect());
            }
            None => Ok(vec![start_block_number]),
        }
    }
}

impl Display for CheckpointRange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let start = match &self.start {
            CheckpointNumberOrTag::Number(number) => number.to_string(),
            _ => self.start.to_string(),
        };

        if let Some(end) = &self.end {
            let end = match end {
                CheckpointNumberOrTag::Number(number) => number.to_string(),
                _ => end.to_string(),
            };
            write!(f, "{}:{}", start, end)
        } else {
            write!(f, "{}", start)
        }
    }
}

pub async fn get_checkpoint_number_from_tag(
    provider: &SuiClient,
    number_or_tag: &CheckpointNumberOrTag,
) -> Result<u64> {
    match number_or_tag {
        CheckpointNumberOrTag::Number(number) => Ok(*number),
        CheckpointNumberOrTag::Earliest => Ok(0),
        CheckpointNumberOrTag::Latest => match provider
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await
        {
            Ok(number) => Ok(number),
            Err(_) => Err(CheckpointRangeError::UnableToFetchCheckpointNumber(
                number_or_tag.clone(),
            )
            .into()),
        },
    }
}
