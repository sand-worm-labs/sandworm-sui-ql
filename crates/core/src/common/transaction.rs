use super::{
    checkpoint::{CheckpointId, CheckpointRange},
    entity_id::{parse_checkpoint_number_or_tag, EntityIdError},
    filters::{
        ComparisonFilterError, EqualityFilter, EqualityFilterError, Filter, FilterError, FilterType,
    },
    query_result::TransactionQueryRes,
};
use crate::interpreter::frontend::parser::Rule;
use alloy::{
    hex::FromHexError,
    primitives::{Address, AddressError, U256},
};
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use sui_types::base_types::SuiAddress;
use sui_types::digests::TransactionDigest;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    ids: Option<Vec<TransactionDigest>>,
    filters: Option<Vec<TransactionFilter>>,
    fields: Vec<TransactionField>,
}

impl Transaction {
    pub fn new(
        ids: Option<Vec<TransactionDigest>>,
        filters: Option<Vec<TransactionFilter>>,
        fields: Vec<TransactionField>,
    ) -> Self {
        Self {
            ids,
            filters,
            fields,
        }
    }

    pub fn ids(&self) -> Option<&Vec<TransactionDigest>> {
        self.ids.as_ref()
    }

    pub fn fields(&self) -> &Vec<TransactionField> {
        &self.fields
    }

    pub fn filters(&self) -> Option<&Vec<TransactionFilter>> {
        self.filters.as_ref()
    }

    pub fn get_checkpoint_id_filter(&self) -> Result<&CheckpointId, TransactionFilterError> {
        self.filters
            .as_ref()
            .and_then(|filters| {
                filters
                    .iter()
                    .find(|f| matches!(f, TransactionFilter::CheckpointId(_)))
                    .and_then(|filter| filter.as_checkpoint_id().ok())
            })
            .ok_or(TransactionFilterError::InvalidBlockIdFilter)
    }

    pub fn filter(&self, tx: &TransactionQueryRes) -> bool {
        if let Some(filters) = &self.filters {
            filters.iter().all(|filter| match filter {
                TransactionFilter::Kind(t) => t.compare(&tx.kind.clone().unwrap()),
                TransactionFilter::Hash(k) => todo!(),
                TransactionFilter::CheckpointId(_) => true,
                TransactionFilter::Sender(k) => k.compare(&tx.sender.unwrap()),
                TransactionFilter::Recipient(l) => todo!(),
                TransactionFilter::GasBudget(m) => todo!(),
                TransactionFilter::GasPrice(n) => todo!(),
                TransactionFilter::GasUsed(o) => todo!(),
                TransactionFilter::Status(p) => todo!(),
                TransactionFilter::ExecutedEpoch(q) => todo!(),
                TransactionFilter::Checkpoint(r) => todo!(),
                TransactionFilter::TimestampMs(s) => todo!(),
                TransactionFilter::EventCount(u) => todo!(),
                TransactionFilter::SignatureScheme(v) => todo!(),
            })
        } else {
            true
        }
    }

    pub fn has_checkpoint_filter(&self) -> bool {
        match self.filters() {
            Some(filters) => filters
                .iter()
                .any(|f| matches!(f, TransactionFilter::CheckpointId(CheckpointId::Range(_)))),
            None => false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    #[error("Unexpected token {0} for transaction")]
    UnexpectedToken(String),
    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    AddressError(#[from] AddressError),
    #[error(transparent)]
    TransactionFieldError(#[from] TransactionFieldError),
    #[error(transparent)]
    TransactionFilterError(#[from] TransactionFilterError),

    #[error("Unknown transaction error: {0}")]
    Other(#[from] anyhow::Error),
}

impl TryFrom<Pairs<'_, Rule>> for Transaction {
    type Error = TransactionError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut ids: Option<Vec<TransactionDigest>> = None;
        let mut filter: Option<Vec<TransactionFilter>> = None;
        let mut fields: Vec<TransactionField> = vec![];

        for pair in pairs {
            match pair.as_rule() {
                Rule::tx_id => {
                    if let Some(ids) = ids.as_mut() {
                        ids.push(TransactionDigest::from_str(pair.as_str())?);
                    } else {
                        ids = Some(vec![TransactionDigest::from_str(pair.as_str())?]);
                    }
                }
                Rule::tx_filter => {
                    let next_filter = pair.into_inner().next().unwrap();
                    if let Some(filter) = filter.as_mut() {
                        filter.push(TransactionFilter::try_from(next_filter)?);
                    } else {
                        filter = Some(vec![TransactionFilter::try_from(next_filter)?]);
                    }
                }
                Rule::tx_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = TransactionField::all_variants().to_vec();
                            continue;
                        }
                    }
                    fields = inner_pairs
                        .map(|pair| TransactionField::try_from(pair.as_str()))
                        .collect::<Result<Vec<TransactionField>, TransactionFieldError>>()?;
                }
                _ => {
                    return Err(TransactionError::UnexpectedToken(pair.as_str().to_string()));
                }
            }
        }

        Ok(Transaction {
            ids,
            filters: filter,
            fields,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum TransactionField {
    Type,
    Digest,
    Sender,
    Recipient,
    Data,
    GasBudget,
    GasPrice,
    GasUsed,
    Status,
    ExecutedEpoch,
    Checkpoint,
    TimestampMs,
    TotalEvents,
    EventDigests,
    RawTransaction,
    TotalObjectChanges,
    TransactionKind,
    Version,
    SignatureScheme,
    PublicKey,
    Signature,
    Chain,
}

impl std::fmt::Display for TransactionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionField::Type => write!(f, "type"),
            TransactionField::Digest => write!(f, "digest"),
            TransactionField::Sender => write!(f, "sender"),
            TransactionField::Recipient => write!(f, "recipient"),
            TransactionField::Data => write!(f, "data"),
            TransactionField::GasBudget => write!(f, "gas_budget"),
            TransactionField::GasPrice => write!(f, "gas_price"),
            TransactionField::GasUsed => write!(f, "gas_used"),
            TransactionField::Status => write!(f, "status"),
            TransactionField::ExecutedEpoch => write!(f, "executed_epoch"),
            TransactionField::Checkpoint => write!(f, "checkpoint"),
            TransactionField::TimestampMs => write!(f, "timestamp_ms"),
            TransactionField::TotalEvents => write!(f, "total_events"),
            TransactionField::TotalObjectChanges => write!(f, "total_object_changes"),
            TransactionField::TransactionKind => write!(f, "transaction_kind"),
            TransactionField::Version => write!(f, "version"),
            TransactionField::RawTransaction => write!(f, "raw_transaction"),
            TransactionField::EventDigests => write!(f, "event_digests"),
            TransactionField::Chain => write!(f, "chain"),
            TransactionField::Signature => write!(f, "signature"),
            TransactionField::SignatureScheme => write!(f, "signature_scheme"),
            TransactionField::PublicKey => write!(f, "public_key"),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionFieldError {
    #[error("Invalid transaction field: {0}")]
    InvalidTransactionField(String),
}

impl TryFrom<&str> for TransactionField {
    type Error = TransactionFieldError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "type" => Ok(TransactionField::Type),
            "digest" => Ok(TransactionField::Digest),
            "sender" => Ok(TransactionField::Sender),
            "recipient" => Ok(TransactionField::Recipient),
            "data" => Ok(TransactionField::Data),
            "chain" => Ok(TransactionField::Chain),
            "gas_budget" => Ok(TransactionField::GasBudget),
            "gas_price" => Ok(TransactionField::GasPrice),
            "gas_used" => Ok(TransactionField::GasUsed),
            "status" => Ok(TransactionField::Status),
            "executed_epoch" => Ok(TransactionField::ExecutedEpoch),
            "checkpoint" => Ok(TransactionField::Checkpoint),
            "timestamp_ms" => Ok(TransactionField::TimestampMs),
            "total_events" => Ok(TransactionField::TotalEvents),
            "event_digests" => Ok(TransactionField::EventDigests),
            "raw_transaction" => Ok(TransactionField::RawTransaction),
            "total_object_changes" => Ok(TransactionField::TotalObjectChanges),
            "transaction_kind" => Ok(TransactionField::TransactionKind),
            "version" => Ok(TransactionField::Version),
            "signature_scheme" => Ok(TransactionField::SignatureScheme),
            "public_key" => Ok(TransactionField::PublicKey),
            "signature" => Ok(TransactionField::Signature),
            invalid_field => Err(TransactionFieldError::InvalidTransactionField(
                invalid_field.to_string(),
            )),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransactionFilterError {
    #[error("Invalid transaction filter property: {0}")]
    InvalidTransactionFilterProperty(String),
    #[error("Missing operator in filter")]
    MissingOperator,
    #[error(transparent)]
    EntityIdError(#[from] EntityIdError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error("BlockId filter is not valid")]
    InvalidBlockIdFilter,
    #[error(transparent)]
    ComparisonFilterError(#[from] ComparisonFilterError),
    #[error(transparent)]
    FilterError(#[from] FilterError),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionFilter {
    Kind(EqualityFilter<String>),
    Hash(EqualityFilter<TransactionDigest>),
    Sender(FilterType<SuiAddress>),
    Recipient(FilterType<SuiAddress>),
    GasBudget(FilterType<u64>),
    GasPrice(FilterType<u64>),
    GasUsed(FilterType<u64>),
    Status(EqualityFilter<bool>),
    ExecutedEpoch(FilterType<u64>),
    Checkpoint(FilterType<u64>),
    TimestampMs(FilterType<u64>),
    EventCount(FilterType<u64>), // Number of events emitted
    SignatureScheme(EqualityFilter<u8>),
    CheckpointId(CheckpointId),
}

impl TransactionFilter {
    pub fn as_checkpoint_id(&self) -> Result<&CheckpointId, TransactionFilterError> {
        if let TransactionFilter::CheckpointId(checkpoint_id) = self {
            Ok(checkpoint_id)
        } else {
            Err(TransactionFilterError::InvalidBlockIdFilter)
        }
    }

    // Helper function to parse filter components
    fn parse_filter<'a, T, F>(
        pair: Pair<'a, Rule>,
        value_parser: F,
        constructor: impl FnOnce(FilterType<T>) -> TransactionFilter,
    ) -> Result<TransactionFilter, TransactionFilterError>
    where
        F: FnOnce(&str) -> T,
        FilterType<T>: TryFrom<(Pair<'a, Rule>, T), Error = FilterError>,
    {
        let mut inner_pair = pair.into_inner();
        let operator = inner_pair.next();

        match operator {
            Some(op) => {
                let value = value_parser(inner_pair.as_str().trim());
                Ok(constructor(FilterType::try_from((op, value))?))
            }
            None => Err(TransactionFilterError::MissingOperator),
        }
    }

    /// Helper function to parse equality filter components
    fn parse_equality_filter<'a, T, F>(
        pair: Pair<'a, Rule>,
        value_parser: F,
        constructor: impl FnOnce(EqualityFilter<T>) -> TransactionFilter,
    ) -> Result<TransactionFilter, TransactionFilterError>
    where
        F: FnOnce(&str) -> T,
        EqualityFilter<T>: TryFrom<(Pair<'a, Rule>, T), Error = EqualityFilterError>,
    {
        let mut inner_pair = pair.into_inner();
        let operator = inner_pair.next();

        match operator {
            Some(op) => {
                let value = value_parser(inner_pair.as_str().trim());
                let filter =
                    EqualityFilter::try_from((op, value)).map_err(|e| FilterError::from(e))?;
                Ok(constructor(filter))
            }
            None => Err(TransactionFilterError::MissingOperator),
        }
    }
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, TransactionFilterError> {
        match pair.as_rule() {
            Rule::checkpointrange_filter => {
                let range = pair
                    .as_str()
                    .trim_start_matches("checkpoint ")
                    .trim_start_matches(|c: char| c.is_whitespace() || c == '=')
                    .trim();

                let (start, end) = match range.split_once(':') {
                    // if ":" is present, we have a start and an end
                    Some((start, end)) => (
                        parse_checkpoint_number_or_tag(start)?,
                        Some(parse_checkpoint_number_or_tag(end)?),
                    ),
                    // else we only have start
                    None => (parse_checkpoint_number_or_tag(range)?, None),
                };
                Ok(TransactionFilter::CheckpointId(CheckpointId::Range(
                    CheckpointRange::new(start, end),
                )))
            }
            Rule::type_filter_type => Self::parse_equality_filter(
                pair,
                |s| s.parse::<String>().unwrap(),
                TransactionFilter::Kind,
            ),
            Rule::sender_filter_type => Self::parse_filter(
                pair,
                |s| SuiAddress::from_str(s).unwrap(),
                TransactionFilter::Sender,
            ),
            Rule::recipient_filter_type => Self::parse_filter(
                pair,
                |s| SuiAddress::from_str(s).unwrap(),
                TransactionFilter::Recipient,
            ),
            Rule::gas_price_filter_type => Self::parse_filter(
                pair,
                |s| s.parse::<u64>().unwrap(),
                TransactionFilter::GasPrice,
            ),
            Rule::gas_budget_filter_type => Self::parse_filter(
                pair,
                |s| s.parse::<u64>().unwrap(),
                TransactionFilter::GasBudget,
            ),
            Rule::gas_used_filter_type => Self::parse_filter(
                pair,
                |s| s.parse::<u64>().unwrap(),
                TransactionFilter::GasUsed,
            ),
            Rule::executed_epoch_filter_type => Self::parse_filter(
                pair,
                |s| s.parse::<u64>().unwrap(),
                TransactionFilter::ExecutedEpoch,
            ),
            Rule::status_filter_type => {
                let mut inner_pair = pair.into_inner();
                let operator = inner_pair.next().unwrap();
                let value = inner_pair.as_str().trim();

                Ok(TransactionFilter::Status(
                    EqualityFilter::try_from((operator, value == "true")).unwrap(),
                ))
            }
            Rule::timestamp_ms_filter_type => Self::parse_filter(
                pair,
                |s| s.parse::<u64>().unwrap(),
                TransactionFilter::TimestampMs,
            ),
            _ => Err(TransactionFilterError::InvalidTransactionFilterProperty(
                pair.as_str().to_string(),
            )),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use alloy::eips::BlockNumberOrTag;

//     use super::*;
//     use crate::common::filters::ComparisonFilter;

//     #[test]
//     fn test_return_true_if_tx_passes_all_filters() {
//         let value = U256::from(1000000000);

//         let tx_query_res = TransactionQueryRes {
//             value: Some(value),
//             ..Default::default()
//         };

//         let transaction = Transaction::new(
//             None,
//             Some(vec![TransactionFilter::Value(FilterType::Comparison(
//                 ComparisonFilter::Lte(value),
//             ))]),
//             vec![TransactionField::Hash],
//         );

//         assert_eq!(true, transaction.filter(&tx_query_res));
//     }

//     #[test]
//     fn test_return_false_if_tx_does_not_pass_any_filters() {
//         let tx_query_res = TransactionQueryRes {
//             value: Some(U256::from(1)),
//             r#type: Some(2),
//             ..Default::default()
//         };

//         // let filter = FilterType::Comparison(ComparisonFilter::Gte(U256::from(1000000000)));

//         // GET type FROM tx WHERE block = 45087:45187, type = 4 ON mekong
//         let transaction = Transaction::new(
//             None,
//             Some(vec![
//                 TransactionFilter::BlockId(BlockId::Range(BlockRange::new(
//                     BlockNumberOrTag::Number(45087),
//                     Some(BlockNumberOrTag::Number(45187)),
//                 ))),
//                 TransactionFilter::Type(EqualityFilter::Eq(4)),
//             ]),
//             vec![TransactionField::Type],
//         );

//         assert_eq!(false, transaction.filter(&tx_query_res));
//     }
// }
