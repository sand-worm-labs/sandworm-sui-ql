use std::error::Error;
use std::sync::Arc;

use super::{
    dump::{Dump, DumpFormat},
    query_result::ExpressionResult,
};
use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use serde::Serialize;

use csv::WriterBuilder;

pub(crate) fn dump_results(result: &ExpressionResult, dump: &Dump) -> Result<(), Box<dyn Error>> {
    match dump.format {
        DumpFormat::Json => {
            let content = serialize_json(result)?;
            std::fs::write(dump.path(), content)?;
        }
        DumpFormat::Csv => {
            let content = match result {
                ExpressionResult::Account(accounts) => serialize_csv(accounts)?,
                ExpressionResult::Checkpoint(blocks) => serialize_csv(blocks)?,
                ExpressionResult::Transaction(txs) => serialize_csv(txs)?,
                ExpressionResult::Coin(coins) => serialize_csv(coins)?,
                ExpressionResult::Object(object) => serialize_csv(object)?,
            };

            std::fs::write(dump.path(), content)?;
        }
        DumpFormat::Parquet => {
            let content = serialize_parquet(result)?;
            std::fs::write(dump.path(), content)?;
        }
    }
    Ok(())
}

fn serialize_json<T: Serialize>(result: &T) -> Result<String, Box<dyn Error>> {
    Ok(serde_json::to_string_pretty(result)?)
}

fn serialize_csv<T: Serialize>(results: &Vec<T>) -> Result<String, Box<dyn Error>> {
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(vec![]);

    for result in results {
        writer.serialize(result)?
    }

    Ok(String::from_utf8(writer.into_inner()?)?)
}

fn serialize_parquet(result: &ExpressionResult) -> Result<Vec<u8>, Box<dyn Error>> {
    let (schema, data) = match result {
        ExpressionResult::Account(accounts) => create_parquet_schema_and_data(accounts)?,
        ExpressionResult::Checkpoint(blocks) => create_parquet_schema_and_data(blocks)?,
        ExpressionResult::Transaction(transactions) => {
            create_parquet_schema_and_data(transactions)?
        }
        ExpressionResult::Coin(coins) => create_parquet_schema_and_data(coins)?,
        ExpressionResult::Object(objects) => create_parquet_schema_and_data(objects)?,
    };

    let batch = RecordBatch::try_new(Arc::new(schema), data)?;

    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), None)?;

    writer.write(&batch)?;
    writer.close()?;

    Ok(buf)
}

fn create_parquet_schema_and_data<T: Serialize>(
    items: &[T],
) -> Result<(Schema, Vec<ArrayRef>), Box<dyn Error>> {
    let mut fields = Vec::new();
    let mut data = Vec::new();

    if let Some(first_item) = items.first() {
        let value = serde_json::to_value(first_item)?;
        if let serde_json::Value::Object(map) = value {
            for (key, _) in map {
                let field = Field::new(&key, DataType::Utf8, true);
                fields.push(field);

                let column_data: Vec<Option<String>> = items
                    .iter()
                    .map(|item| {
                        let item_value = serde_json::to_value(item).unwrap();
                        if let serde_json::Value::Object(item_map) = item_value {
                            item_map.get(&key).and_then(|v| Some(v.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();

                data.push(Arc::new(StringArray::from(column_data)) as ArrayRef);
            }
        }
    }

    let schema = Schema::new(fields);
    Ok((schema, data))
}

#[cfg(test)]
mod test {
    use super::{serialize_csv, serialize_json, serialize_parquet};
    use crate::common::query_result::{AccountQueryRes, ExpressionResult};

    #[test]
    fn test_serialize_json() {
        let res = AccountQueryRes {
            address: None,
            chain: None,
            sui_balance: Some(1000),
            coin_owned: None,
            staked_amount: Some(0),
            active_delegations: None,
        };
        let result = ExpressionResult::Account(vec![res]);
        let content = serialize_json(&result).unwrap();

        assert_eq!(content, "{\n  \"account\": [\n    {\n      \"nonce\": 0,\n      \"balance\": \"100\"\n    }\n  ]\n}");
    }

    #[test]
    fn test_serialize_csv() {
        let res = vec![
            AccountQueryRes {
                address: None,
                chain: None,
                sui_balance: Some(1000),
                coin_owned: None,
                staked_amount: Some(0),
                active_delegations: None,
            },
            AccountQueryRes {
                address: None,
                chain: None,
                sui_balance: Some(1000),
                coin_owned: None,
                staked_amount: Some(0),
                active_delegations: None,
            },
        ];
        let content = serialize_csv(&res).unwrap();

        assert_eq!(content, "sui_balance,stake_amount\n1000,0\n1000,0\n");
    }

    #[test]
    fn test_serialize_parquet() {
        let res = AccountQueryRes {
            address: None,
            chain: None,
            sui_balance: Some(1000),
            coin_owned: None,
            staked_amount: Some(0),
            active_delegations: None,
        };
        let result = ExpressionResult::Account(vec![res]);
        let content = serialize_parquet(&result).unwrap();

        // Since Parquet is a binary format, we can't easily assert its content.
        // Instead, we'll just check that we get a non-empty result.
        assert!(!content.is_empty());
    }
}
