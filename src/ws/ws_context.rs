use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc as tmpsc;
use uuid::Uuid;

use crate::ws::{
    default_channel_names::{
        NOTIFICATION_CHANNEL, QUEUED_TASKS_INFO_CHANNEL, RUNNING_PROCESSES, SYS_INFO_CHANNEL_NAME,
    },
    WsMessage,
};

#[derive(Clone)]
pub struct WsContext {
    // map unique client id -> channel to a channel name eg {"asd": {"uuid4": transmitter}}
    pub channels: Arc<DashMap<String, DashMap<Uuid, tmpsc::Sender<WsMessage>>>>,
}

impl WsContext {
    pub fn new() -> Self {
        let channels = DashMap::new();

        // Pre-load default channels with empty HashMaps
        channels.insert(SYS_INFO_CHANNEL_NAME.to_string(), DashMap::new());
        channels.insert(NOTIFICATION_CHANNEL.to_string(), DashMap::new());
        channels.insert(QUEUED_TASKS_INFO_CHANNEL.into(), DashMap::new());
        channels.insert(RUNNING_PROCESSES.into(), DashMap::new());

        return WsContext {
            channels: Arc::new(channels),
        };
    }

    pub fn add_channel(&self, channel_name: &str) {
        self.channels
            .insert(channel_name.to_string(), DashMap::new());
    }

    pub fn remove_channel(&self, channel_name: &str) {
        self.channels.remove(channel_name);
    }
}

pub async fn broadcast_to_channel(ws_context: &WsContext, channel: &str, msg: WsMessage) {
    if let Some(subs) = ws_context.channels.get(channel) {
        let mut dead_clients = vec![];
        for entry in subs.iter() {
            let client_id = entry.key();
            let tx = entry.value();
            if tx.send(msg.clone()).await.is_err() {
                dead_clients.push(*client_id);
            }
        }
        // Cleanup disconnected clients
        for id in dead_clients {
            subs.remove(&id);
        }
    }
}
