// use super::ens::NameOrAddress;
use crate::interpreter::frontend::parser::Rule;
use eql_macros::EnumVariants;
use pest::iterators::{Pair, Pairs};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{write, Display},
    str::FromStr,
};
use sui_json_rpc_types::EventFilter as SuiEventFilter;
use sui_types::digests::TransactionDigest;

#[derive(thiserror::Error, Debug)]
pub enum EventError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error(transparent)]
    EventFieldError(#[from] EventFieldError),

    #[error(transparent)]
    EventFilterError(#[from] EventFilterError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Event {
    id: Option<Vec<String>>,
    filter: Option<Vec<EventFilter>>,
    fields: Vec<EventField>,
}

impl Event {
    pub fn new(
        id: Option<Vec<String>>,
        filter: Option<Vec<EventFilter>>,
        fields: Vec<EventField>,
    ) -> Self {
        Self { id, filter, fields }
    }

    pub fn ids(&self) -> Option<&Vec<String>> {
        self.id.as_ref()
    }

    pub fn filter(&self) -> Option<Vec<EventFilter>> {
        self.filter.clone()
    }

    pub fn fields(&self) -> Vec<EventField> {
        self.fields.clone()
    }
}

impl TryFrom<Pairs<'_, Rule>> for Event {
    type Error = EventError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        let mut fields: Vec<EventField> = vec![];
        let mut id: Option<Vec<String>> = None;
        let mut filter: Option<Vec<EventFilter>> = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::event_fields => {
                    let inner_pairs = pair.into_inner();

                    if let Some(pair) = inner_pairs.peek() {
                        if pair.as_rule() == Rule::wildcard {
                            fields = EventField::all_variants().to_vec();
                            continue;
                        }
                    }

                    fields = inner_pairs
                        .map(|pair| EventField::try_from(pair))
                        .collect::<Result<Vec<EventField>, EventFieldError>>()?;
                }
                // Rule::tx_id => {
                //     if let Some(id) = id.as_mut() {
                //         id.push(NameOrAddress::from_str(pair.as_str())?);
                //     } else {
                //         id = Some(vec![NameOrAddress::from_str(pair.as_str())?]);
                //     }
                // }
                // Rule::coin_filter_list => {
                //     filter = Some(
                //         pair.into_inner()
                //             .map(|pair| EventFilter::try_from(pair))
                //             .collect::<Result<Vec<EventFilter>, EventFilterError>>()?,
                //     );
                // }
                _ => {
                    return Err(EventError::UnexpectedToken(pair.as_str().to_string()));
                }
            }
        }

        Ok(Event { id, filter, fields })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EventFilterError {
    #[error("Unexpected token {0} for Event filter")]
    UnexpectedToken(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EventFilter {
    EventId(String),
    Transaction(TransactionDigest),
}

// impl TryFrom<Pair<'_, Rule>> for EventFilter {
//     type Error = EventFilterError;

//     fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
//         match pair.as_rule() {
//             Rule::address_filter => {
//                 let address = NameOrAddress::from_str(pair.as_str())?;
//                 Ok(EventFilter::Address(address))
//             }
//             _ => {
//                 return Err(EventFilterError::UnexpectedToken(pair.as_str().to_string()));
//             }
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumVariants)]
pub enum EventField {
    TxDigest,
    EventSeq,
    PackageId,
    Module,
    Sender,
    EventType,
    BcsEncoding,
    Bcs,
}

impl Display for EventField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventField::TxDigest => "tx_digest",
            EventField::EventSeq => "event_seq",
            EventField::PackageId => "package_id",
            EventField::Module => "module",
            EventField::Sender => "sender",
            EventField::EventType => "event_type",
            EventField::BcsEncoding => "bcs_encoding",
            EventField::Bcs => "bcs",
        };
        write!(f, "{}", s)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EventFieldError {
    #[error("Invalid field for entity Event: {0}")]
    InvalidField(String),
}

impl<'a> TryFrom<Pair<'a, Rule>> for EventField {
    type Error = EventFieldError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        EventField::try_from(pair.as_str())
    }
}

impl TryFrom<&str> for EventField {
    type Error = EventFieldError;

    fn try_from(value: &str) -> Result<Self, EventFieldError> {
        match value {
            "tx_digest" => Ok(EventField::TxDigest),
            "event_seq" => Ok(EventField::EventSeq),
            "package_id" => Ok(EventField::PackageId),
            "module" => Ok(EventField::Module),
            "sender" => Ok(EventField::Sender),
            "event_type" => Ok(EventField::EventType),
            "bcs_encoding" => Ok(EventField::BcsEncoding),
            "bcs" => Ok(EventField::Bcs),
            _ => Err(EventFieldError::InvalidField(value.to_string())),
        }
    }
}
