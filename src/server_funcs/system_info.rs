use leptos::prelude::*;

use crate::models::sys_info::SysInfo;

#[server]
pub async fn get_system_info() -> Result<SysInfo, ServerFnError> {
    use crate::server_only::system_info::SysInfoCollector;
    let sys_info_collector = expect_context::<SysInfoCollector>();
    Ok(sys_info_collector.view_sys_info().await)
}
