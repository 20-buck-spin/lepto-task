use crate::{
    models::sys_info::SysInfo,
    ws::{
        default_channel_names::SYS_INFO_CHANNEL_NAME,
        ws_context::{broadcast_to_channel, WsContext},
        WsMessage,
    },
};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Disks, Networks, System};
use tokio::sync::RwLock;

const REFRESH_SECONDS: u64 = 5;

#[derive(Clone)]
pub struct SysInfoCollector {
    pub sys: Arc<RwLock<System>>,
    pub networks: Arc<RwLock<Networks>>,
    pub disks: Arc<RwLock<Disks>>,
}

impl SysInfoCollector {
    pub fn new() -> Self {
        let sys = Arc::new(RwLock::new(System::new()));
        let networks = Arc::new(RwLock::new(Networks::new_with_refreshed_list()));
        let disks = Arc::new(RwLock::new(Disks::new_with_refreshed_list()));
        SysInfoCollector {
            sys,
            networks,
            disks,
        }
    }

    pub async fn view_sys_info(&self) -> SysInfo {
        let (cpu_usage, ram_used, ram_total, swap_total, swap_used) = {
            let sys_read_lock = self.sys.read().await;

            let cpu_usage = sys_read_lock
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect::<Vec<f32>>();

            let ram_used = sys_read_lock.used_memory();
            let ram_total = sys_read_lock.total_memory();
            let swap_total = sys_read_lock.total_swap();
            let swap_used = sys_read_lock.used_swap();

            (cpu_usage, ram_used, ram_total, swap_total, swap_used)
        };

        //network
        let (network_up, network_down) = {
            let network_read_lock = self.networks.read().await;
            let mut network_up = 0;
            let mut network_down = 0;
            for (_, network) in network_read_lock.iter() {
                network_down += network.received() / REFRESH_SECONDS;
                network_up += network.transmitted() / REFRESH_SECONDS;
            }
            (network_up, network_down)
        };

        //disk info
        let (disk_write, disk_read) = {
            let disk_read_lock = self.disks.read().await;
            let mut disk_write = 0;
            let mut disk_read = 0;
            for disk in disk_read_lock.list() {
                let usage = disk.usage();
                disk_write += usage.written_bytes / REFRESH_SECONDS;
                disk_read += usage.read_bytes / REFRESH_SECONDS;
            }
            (disk_write, disk_read)
        };

        SysInfo {
            cpu_usage,
            ram_total,
            ram_used,
            swap_total,
            swap_used,
            network_down,
            network_up,
            disk_read,
            disk_write,
        }
    }

    async fn update_sys_info_loop(&mut self) {
        loop {
            tokio::time::sleep(Duration::from_secs(REFRESH_SECONDS)).await;

            let mut sys_write_lock = self.sys.write().await;
            let mut networks_write_lock = self.networks.write().await;
            let mut disks_write_lock = self.disks.write().await;

            sys_write_lock.refresh_all();
            networks_write_lock.refresh(true);
            disks_write_lock.refresh(true);
        }
    }

    async fn broadcast_sys_info_loop(&self, ws_context: WsContext) -> () {
        loop {
            tokio::time::sleep(Duration::from_secs(REFRESH_SECONDS)).await;
            let sys_info = self.view_sys_info().await;
            match serde_json::to_value(sys_info) {
                Ok(payload) => {
                    let message = WsMessage::Data {
                        channel: SYS_INFO_CHANNEL_NAME.into(),
                        payload,
                    };

                    broadcast_to_channel(&ws_context, SYS_INFO_CHANNEL_NAME, message).await;
                }
                Err(e) => {
                    // add logging
                    println!("Failed to broadcase. Err {}", e)
                }
            }
        }
    }

    pub async fn init(&self, ws_context: WsContext) {
        let mut self_clone = self.clone();
        tokio::task::spawn(async move { self_clone.update_sys_info_loop().await });
        let self_clone = self.clone();
        tokio::task::spawn(async move { self_clone.broadcast_sys_info_loop(ws_context).await });
    }
}
