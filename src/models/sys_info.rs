use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct SysInfo {
    pub cpu_usage: Vec<f32>,
    pub ram_used: u64,
    pub ram_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub network_up: u64,
    pub network_down: u64,
    pub disk_read: u64,
    pub disk_write: u64,
}
