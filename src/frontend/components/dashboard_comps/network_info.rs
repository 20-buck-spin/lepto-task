use crate::models::sys_info::SysInfo;
use crate::utils::format_bytes;
use leptos::prelude::*;
#[component]
pub fn NetworkInfo(sys_info: Signal<SysInfo>) -> impl IntoView {
    view! {
       <div class="dashboard-card-kv-container">
         <div class="dashboard-card-key">
            <span>"Up"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().network_up)}</span>
         </div>
         <div class="dashboard-card-key">
            <span>"Down"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().network_down)}</span>
         </div>
       </div>
    }
}
