use sui_ql_core::interpreter::suiql as sul_interpreter;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn suiql(program: &str) -> Result<JsValue, JsValue> {
    let result = sul_interpreter(program).await;

    match result {
        Ok(result) => {
            let result = serde_wasm_bindgen::to_value(&result)?;
            return Ok(result);
        }
        Err(e) => {
            return Err(JsValue::from_str(&e.to_string()));
        }
    }
}
