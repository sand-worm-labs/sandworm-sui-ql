use super::{
    resolve_account::resolve_account_query, resolve_checkpoint::resolve_checkpoint_query,
    resolve_coin::resolve_coin_query, resolve_transaction::resolve_transaction_query,resolve_object::resolve_object_query
};
use crate::common::{
    entity::Entity,
    query_result::{ExpressionResult, QueryResult},
    serializer::dump_results,
    types::{Expression, GetExpression},
};
use anyhow::Result;

pub struct ExecutionEngine;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExecutionEngineError {
    #[error("Neither an entity_id nor a filter was provided. Pest rules should have prevented this from happening.")]
    NoEntityIdOrFilter,
    #[error("Multiple filters are not supported for block queries.")]
    MultipleFiltersNotSupported,
}

impl ExecutionEngine {
    pub fn new() -> ExecutionEngine {
        ExecutionEngine
    }

    pub async fn run(&self, expressions: Vec<Expression>) -> Result<Vec<QueryResult>> {
        let mut query_results = vec![];

        for expression in expressions {
            match expression {
                Expression::Get(get_expr) => {
                    let result = self.run_get_expr(&get_expr).await?;
                    query_results.push(QueryResult::new(result));
                }
            }
        }

        Ok(query_results)
    }

    async fn run_get_expr(&self, expr: &GetExpression) -> Result<ExpressionResult> {
        let result = match &expr.entity {
            Entity::Checkpoint(checkpoint) => ExpressionResult::Checkpoint(
                resolve_checkpoint_query(checkpoint, &expr.chains).await?,
            ),
            Entity::Account(account) => {
                ExpressionResult::Account(resolve_account_query(account, &expr.chains).await?)
            }
            Entity::Transaction(transaction) => ExpressionResult::Transaction(
                resolve_transaction_query(transaction, &expr.chains).await?,
            ),
            Entity::Coin(coin) => {
                ExpressionResult::Coin(resolve_coin_query(coin, &expr.chains).await?)
            }
            Entity::Object(object) => {
                ExpressionResult::Object(resolve_object_query(object, &expr.chains).await?)
            },
        };

        if let Some(dump) = &expr.dump {
            let _ = dump_results(&result, dump);
        }

        Ok(result)
    }
}
