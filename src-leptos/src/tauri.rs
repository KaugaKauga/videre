use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

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
