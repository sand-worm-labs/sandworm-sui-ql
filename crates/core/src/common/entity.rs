use super::account::AccountError;
use super::transaction::TransactionError;
use crate::common::{
    account::Account, checkpoint::Checkpoint, checkpoint::CheckpointError, coin::Coin,
    coin::CoinError, transaction::Transaction,
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
    CheckpointError(#[from] CheckpointError),

    #[error(transparent)]
    AccountError(#[from] AccountError),

    #[error(transparent)]
    CoinError(#[from] CoinError),
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    Account(Account),
    Checkpoint(Checkpoint),
    Transaction(Transaction),
    Coin(Coin),
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
                    let checkpoint = Checkpoint::try_from(pair.into_inner())?;
                    return Ok(Entity::Checkpoint(checkpoint));
                }
                Rule::tx_get => {
                    let tx = Transaction::try_from(pair.into_inner())?;
                    return Ok(Entity::Transaction(tx));
                }
                // Rule::coin_get => {
                //     let coin = Coin::try_from(pair.into_inner())?;
                //     return Ok(Entity::Coin(coin));
                // }
                _ => return Err(EntityError::UnexpectedToken(pair.as_str().to_string())),
            }
        }
        Err(EntityError::MissingEntity)
    }
}
