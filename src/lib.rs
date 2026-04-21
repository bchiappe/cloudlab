pub mod app;
pub mod auth;
pub mod hosts;
pub mod vms;
pub mod llms;
pub mod containers;
pub mod databases;
pub mod backups;
pub mod jobs;
pub mod settings;
pub mod proxy;
pub mod ssh;
pub mod components;
pub mod storage;
pub mod storage_ssr;
pub mod api_keys;
pub mod ai_proxy;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub String);

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
