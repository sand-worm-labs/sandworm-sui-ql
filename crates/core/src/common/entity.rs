use super::account::AccountError;
use super::transaction::TransactionError;
use crate::common::{
    account::Account, checkpoint::Checkpoint, checkpoint::CheckpointError, coin::Coin,
    coin::CoinError, object::Object, object::ObjectError, transaction::Transaction,
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

    #[error(transparent)]
    ObjectError(#[from] ObjectError),
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    Account(Account),
    Checkpoint(Checkpoint),
    Transaction(Transaction),
    Coin(Coin),
    Object(Object),
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
                Rule::coin_get => {
                    let coin = Coin::try_from(pair.into_inner())?;
                    return Ok(Entity::Coin(coin));
                }
                Rule::object_get => {
                    let object = Object::try_from(pair.into_inner())?;
                    return Ok(Entity::Object(object));
                }
                _ => return Err(EntityError::UnexpectedToken(pair.as_str().to_string())),
            }
        }
        Err(EntityError::MissingEntity)
    }
}
