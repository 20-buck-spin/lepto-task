use crate::models::sys_info::SysInfo;
use crate::utils::format_bytes;
use leptos::prelude::*;
#[component]
pub fn DiskInfo(sys_info: Signal<SysInfo>) -> impl IntoView {
    view! {
       <div class="dashboard-card-kv-container">
         <div class="dashboard-card-key">
            <span>"Disk read"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().disk_read)}</span>
         </div>
         <div class="dashboard-card-key">
            <span>"Disk write"</span>
         </div>
         <div class="dashboard-card-val">
            <span>{move || format_bytes(sys_info.read().disk_write)}</span>
         </div>
       </div>
    }
}
