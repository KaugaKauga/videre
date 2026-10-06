use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::types::{ConnectionInfo, QueryResult, RelationStructure};

fn get_invoke_fn() -> Result<js_sys::Function, String> {
    let window = web_sys::window().ok_or("no global window")?;

    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "__TAURI__ not found on window — is withGlobalTauri enabled?")?;

    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "__TAURI__.core not found")?;

    let invoke = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "__TAURI__.core.invoke not found")?;

    invoke
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "invoke is not a function".to_string())
}

/// Call a Tauri command and deserialize the result.
pub async fn invoke<T: DeserializeOwned>(cmd: &str, args: impl Serialize) -> Result<T, String> {
    let invoke_fn = get_invoke_fn()?;

    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;

    let promise = invoke_fn
        .call2(&JsValue::NULL, &JsValue::from_str(cmd), &args_js)
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

    let js_result = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

    serde_wasm_bindgen::from_value(js_result).map_err(|e| e.to_string())
}

/// Fetch all column metadata for one relation.
///
/// `schema` and `relation` match the command's explicit relation identity.
pub async fn get_relation_structure(
    relation: &str,
    schema: &str,
) -> Result<RelationStructure, String> {
    invoke(
        "get_relation_structure",
        &serde_json::json!({
            "relation": relation,
            "schema": schema,
        }),
    )
    .await
}

fn execute_query_args(sql: &str, request_id: &str) -> serde_json::Value {
    serde_json::json!({ "sql": sql, "requestId": request_id })
}

fn cancel_query_args(request_id: &str) -> serde_json::Value {
    serde_json::json!({ "requestId": request_id })
}

pub async fn execute_query(sql: &str, request_id: &str) -> Result<QueryResult, String> {
    invoke("execute_query", &execute_query_args(sql, request_id)).await
}

pub async fn cancel_query(request_id: &str) -> Result<(), String> {
    invoke_void("cancel_query", &cancel_query_args(request_id)).await
}

pub async fn get_connection_info() -> Result<ConnectionInfo, String> {
    invoke("get_connection_info", &serde_json::json!({})).await
}

/// Call a Tauri command that returns nothing.
pub async fn invoke_void(cmd: &str, args: impl Serialize) -> Result<(), String> {
    let invoke_fn = get_invoke_fn()?;

    let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;

    let promise = invoke_fn
        .call2(&JsValue::NULL, &JsValue::from_str(cmd), &args_js)
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

    JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn execute_query_sends_entire_sql_and_camel_case_request_id() {
        let sql = "  WITH draft AS (SELECT 'λ' AS value)\nSELECT * FROM draft;\n";
        assert_eq!(
            execute_query_args(sql, "query-123"),
            json!({ "sql": sql, "requestId": "query-123" })
        );
    }

    #[test]
    fn cancel_query_uses_the_same_request_id_contract() {
        assert_eq!(
            cancel_query_args("query-123"),
            json!({ "requestId": "query-123" })
        );
    }
}
