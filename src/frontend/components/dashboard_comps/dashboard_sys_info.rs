use leptos::task::spawn_local;
use leptos::{logging, prelude::*};

use crate::models::sys_info::SysInfo;
use crate::server_funcs::system_info::get_system_info;
use crate::{
    frontend::components::dashboard_comps::{
        card_loaders::CardLoader, cpu_info::CpuInfoCard, disk_info::DiskInfo,
        network_info::NetworkInfo, ram_info::RamInfo,
    },
    ws::{default_channel_names::SYS_INFO_CHANNEL_NAME, WsClientContext, WsMessage},
};

#[component]
pub fn SystemInfo() -> impl IntoView {
    let system_info: RwSignal<Option<SysInfo>> = RwSignal::new(None);
    let ws = expect_context::<WsClientContext>();

    Effect::new(move |_| {
        untrack(move || {
            // fetch system_info on load
            spawn_local(async move {
                match get_system_info().await {
                    Ok(info) => {
                        system_info.set(Some(info));
                    }
                    Err(err) => {
                        logging::error!("Failed to fetch initial system info: {:?}", err);
                    }
                }
            });
            // Send sub message to ws
            ws.send.run(WsMessage::Subscribe {
                channel: SYS_INFO_CHANNEL_NAME.into(),
            });
        });
    });
    on_cleanup(move || {
        ws.send.run(WsMessage::Unsubscribe {
            channel: SYS_INFO_CHANNEL_NAME.into(),
        });
    });
    Effect::new(move |_| {
        if let Some(msg) = ws.latest_message.get() {
            match msg {
                WsMessage::Data { channel, payload } if &channel == SYS_INFO_CHANNEL_NAME => {
                    match serde_json::from_value::<SysInfo>(payload.clone()) {
                        Ok(sys_info) => *system_info.write() = Some(sys_info),
                        Err(e) => {
                            leptos::logging::error!(
                                "Failed to deserialize system info message. Message: {} err {}",
                                payload,
                                e
                            )
                        }
                    }
                }
                WsMessage::SubscribeFail { channel, reason }
                    if &channel == SYS_INFO_CHANNEL_NAME =>
                {
                    leptos::logging::error!("Sub failed for {}: {}", channel, reason);
                }
                _ => {}
            }
        }
    });
    let is_loaded = Signal::derive(move || system_info.with(|sys| sys.is_some()));
    let safe_sys = Signal::derive(move || system_info.get().unwrap_or_default());

    view!(
        <div class="sys-info-cards-container">
            <div class="sys-info-card">
                <div class="sys-info-card-header-container">
                    <span class="sys-info-card-header">"CPU"</span>
                    <img src="/icons/cpuicon.png" alt="Star"/>
                </div>
                <Show fallback = view!({CardLoader})
                    when = move || is_loaded.get()
                >
                <CpuInfoCard sys_info=safe_sys/>
                </Show>
            </div>
            <div class="sys-info-card">
                <div class="sys-info-card-header-container">
                    <span class="sys-info-card-header">"RAM"</span>
                    <img src="/icons/ramicon.png" alt="Star"/>
                </div>
                <Show fallback = view!({CardLoader})
                    when = move || is_loaded.get()
                >
                <RamInfo sys_info=safe_sys/>
                </Show>
            </div>
            <div class="sys-info-card">
                <div class="sys-info-card-header-container">
                    <span class="sys-info-card-header">"Network"</span>
                    <img src="/icons/networkicon.png" alt="Star"/>
                </div>
                <Show fallback = view!({CardLoader})
                    when = move || is_loaded.get()
                >
                <NetworkInfo sys_info=safe_sys/>
                </Show>
            </div>
            <div class="sys-info-card">
                <div class="sys-info-card-header-container">
                    <span class="sys-info-card-header">"Disk"</span>
                    <img src="/icons/diskicon.png" alt="Star"/>
                </div>
                <Show fallback = view!({CardLoader})
                    when = move || is_loaded.get()
                >
                <DiskInfo sys_info=safe_sys/>
                </Show>
            </div>
        </div>
    )
}
