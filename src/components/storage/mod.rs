pub mod file_browser;
pub mod storage_list;

use leptos::prelude::*;
pub use storage_list::StoragePage;

#[component]
pub fn StorageList() -> impl IntoView {
    view! { <storage_list::StoragePage /> }
}
