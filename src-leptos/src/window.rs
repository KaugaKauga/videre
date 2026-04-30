//! Typed Tauri window operations.
//!
//! Thin wrappers around `__TAURI__.window.getCurrentWindow()` methods.
//! Each function is locked to a specific operation — no arbitrary method
//! dispatch.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Resolve `window.__TAURI__.window.getCurrentWindow()` → JS object.
fn get_current_window() -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("no global window")?;

    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "__TAURI__ not found on window")?;

    let window_mod = js_sys::Reflect::get(&tauri, &JsValue::from_str("window"))
        .map_err(|_| "__TAURI__.window not found")?;

    let get_current_fn = js_sys::Reflect::get(&window_mod, &JsValue::from_str("getCurrentWindow"))
        .map_err(|_| "getCurrentWindow not found")?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "getCurrentWindow is not a function")?;

    get_current_fn
        .call0(&JsValue::NULL)
        .map_err(|e| format!("getCurrentWindow() failed: {e:?}"))
}

/// Call a zero-arg method on the current window and await its promise.
async fn call(method: &str) -> Result<(), String> {
    let win = get_current_window()?;

    let method_fn = js_sys::Reflect::get(&win, &JsValue::from_str(method))
        .map_err(|_| format!("{method} not found on window"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("{method} is not a function"))?;

    let promise = method_fn
        .call0(&win)
        .map_err(|e| format!("{method}() failed: {e:?}"))?;

    JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| format!("{method} promise rejected: {e:?}"))?;

    Ok(())
}

/// Log an error string to the browser console.
fn log_err(msg: &str) {
    web_sys::console::error_1(&JsValue::from_str(msg));
}

// ---------------------------------------------------------------------------
// Public API — one function per permitted operation
// ---------------------------------------------------------------------------

pub async fn minimize() {
    if let Err(e) = call("minimize").await {
        log_err(&format!("[videre::window] minimize failed: {e}"));
    }
}

pub async fn toggle_maximize() {
    if let Err(e) = call("toggleMaximize").await {
        log_err(&format!("[videre::window] toggleMaximize failed: {e}"));
    }
}

pub async fn close() {
    if let Err(e) = call("close").await {
        log_err(&format!("[videre::window] close failed: {e}"));
    }
}

pub async fn start_dragging() {
    if let Err(e) = call("startDragging").await {
        log_err(&format!("[videre::window] startDragging failed: {e}"));
    }
}
