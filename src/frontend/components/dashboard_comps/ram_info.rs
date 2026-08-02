use crate::models::sys_info::SysInfo;
use crate::utils::format_bytes;
use leptos::prelude::*;
#[component]
pub fn RamInfo(sys_info: Signal<SysInfo>) -> impl IntoView {
    view! {
       <div class="dashboard-card-kv-container">
         <div class="dashboard-card-key">
            <span>"Used"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().ram_used)}</span>
         </div>
         <div class="dashboard-card-key">
            <span>"Total"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().ram_total)}</span>
         </div>
         <div class="dashboard-card-key">
            <span>"Used swap"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().swap_used)}</span>
         </div>
         <div class="dashboard-card-key">
            <span>"Total swap"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().swap_total)}</span>
         </div>
       </div>
    }
}
