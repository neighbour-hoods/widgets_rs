mod app;
pub mod holochain_client_wrapper;
mod myclass;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::Model>();
    Ok(())
}
