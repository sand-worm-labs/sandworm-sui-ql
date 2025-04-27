use super::account::AccountError;
use super::transaction::TransactionError;
use crate::common::{
    account::Account,
    checkpoint::Checkpoint,
    checkpoint::CheckpointError, 
    transaction::Transaction,
};
use crate::interpreter::frontend::parser::Rule;
use pest::iterators::Pairs;

#[derive(thiserror::Error, Debug)]
pub enum EntityError {
    #[error("Unexpected token {0}")]
    UnexpectedToken(String),

    #[error("Missing entity")]
    MissingEntity,

    #[error(transparent)]
    TransactionError(#[from] TransactionError),

    #[error(transparent)]
    LogsError(#[from] LogsError),

    #[error(transparent)]
    CheckpointError(#[from] CheckpointError),

    #[error(transparent)]
    AccountError(#[from] AccountError),
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    Account(Account),
    Checkpoint(Checkpoint),
    Transaction(Transaction),
    Logs(Logs),
}

impl TryFrom<Pairs<'_, Rule>> for Entity {
    type Error = EntityError;

    fn try_from(pairs: Pairs<'_, Rule>) -> Result<Self, Self::Error> {
        for pair in pairs {
            match pair.as_rule() {
                Rule::account_get => {
                    let account = Account::try_from(pair.into_inner())?;
                    return Ok(Entity::Account(account));
                }
                Rule::checkpoint_get => {
                    let block = Block::try_from(pair.into_inner())?;
                    return Ok(Entity::Block(block));
                }
                Rule::tx_get => {
                    let tx = Transaction::try_from(pair.into_inner())?;
                    return Ok(Entity::Transaction(tx));
                }
                _ => return Err(EntityError::UnexpectedToken(pair.as_str().to_string())),
            }
        }
        Err(EntityError::MissingEntity)
    }
}
