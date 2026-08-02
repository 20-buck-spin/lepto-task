#![recursion_limit = "512"]
pub mod app;
pub mod frontend;
pub mod models;
pub mod server_funcs;
pub mod utils;
pub mod ws;

#[cfg(feature = "ssr")]
pub mod server_only;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
