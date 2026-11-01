use std::time::Duration;

use chrono::{DateTime, Utc};
use leptos::logging;
use serde_json::Value;
use sqlx::{Pool, QueryBuilder, Sqlite};
use tokio::sync::mpsc;

use crate::{
    models::task_models::{LogEntry, LogLevel},
    utils::get_run_logs_channel_name,
    ws::{
        ws_context::{broadcast_to_channel, WsContext},
        WsMessage,
    },
};

#[derive(Clone)]
pub struct LogHandler {
    tx: mpsc::Sender<(i64, LogEntry)>,
    ws_context: WsContext,
}

impl LogHandler {
    pub fn new(db: Pool<Sqlite>, ws_context: WsContext) -> Self {
        let (tx, mut rx) = mpsc::channel::<(i64, LogEntry)>(10_000);

        tokio::spawn(async move {
            let mut batch: Vec<(i64, LogEntry)> = Vec::with_capacity(500);
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                tokio::select! {
                    maybe = rx.recv() => {
                        match maybe {
                            Some(entry) => {
                                batch.push(entry);
                                if batch.len() >= 500 {
                                    LogHandler::flush_logs(&db, &mut batch).await;
                                }
                            },
                            None => {
                                    LogHandler::flush_logs(&db, &mut batch).await;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty(){
                            LogHandler::flush_logs(&db, &mut batch).await;
                        }
                    }

                }
            }
        });

        Self { tx, ws_context }
    }

    pub async fn insert_log(&self, run_id: i64, log_entry: LogEntry) {
        if let Err(err) = self.tx.send((run_id, log_entry)).await {
            // channel closed; fall back to direct write or log error
            logging::error!("Log handling channel closed. Captured err {}", err)
        }
    }

    async fn flush_logs(db: &Pool<Sqlite>, batch: &mut Vec<(i64, LogEntry)>) {
        let Ok(mut tx) = db.begin().await.inspect_err(|err| {
            logging::error!("Failed to open transaction on flush logs. Err {}", err)
        }) else {
            return;
        };

        for chunk in batch.chunks(500) {
            let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                "
                INSERT INTO runlog
                (run_id, created_dt, level, message, raw_payload)
                ",
            );

            query_builder.push_values(chunk, |mut b, (run_id, log_entry)| {
                b.push_bind(run_id);
                b.push_bind(log_entry.timestamp);
                b.push_bind(log_entry.level.to_string());
                b.push_bind(&log_entry.message);
                b.push_bind(&log_entry.raw_payload);
            });

            let query = query_builder.build();
            let _ = query
                .execute(&mut *tx)
                .await
                .inspect_err(|err| logging::error!("Failed to add log chunk to db. Err {}", err));
        }
        let _ = tx.commit().await.inspect_err(|err| {
            logging::error!("Failed to commit log flush transaction. Err {}", err)
        });
    }

    fn extract_level(map: &serde_json::Map<String, Value>) -> LogLevel {
        map.get("level")
            .or_else(|| map.get("severity"))
            .or_else(|| map.get("lvl"))
            .map(|lvl| {
                // 1. Handle String-based log levels
                if let Some(lvl_str) = lvl.as_str() {
                    match lvl_str.to_uppercase().as_str() {
                        "TRACE" | "DEBUG" => LogLevel::Debug,
                        "INFO" => LogLevel::Info,
                        "WARN" | "WARNING" => LogLevel::Warn,
                        "ERROR" => LogLevel::Error,
                        "CRITICAL" | "FATAL" | "EMERGENCY" => LogLevel::Critical,
                        _ => LogLevel::Info, // Unknown strings default to Info
                    }
                }
                // 2. Handle Integer-based log levels (e.g., Pino)
                else if let Some(lvl_num) = lvl.as_i64() {
                    match lvl_num {
                        10 | 20 => LogLevel::Debug,
                        30 => LogLevel::Info,
                        40 => LogLevel::Warn,
                        50 => LogLevel::Error,
                        60..=100 => LogLevel::Critical,
                        _ => LogLevel::Info, // Unknown numbers default to Info
                    }
                }
                // 3. Fallback for invalid types
                else {
                    LogLevel::Info
                }
            })
            .unwrap_or(LogLevel::Info) // Fallback if no level key exists
    }

    fn parse_log(log_str: String) -> LogEntry {
        let now = Utc::now();
        match serde_json::from_str::<Value>(&log_str) {
            Ok(Value::Object(map)) => {
                // --- Extract Message ---
                let message = if let Some(msg) = map.get("message").and_then(Value::as_str) {
                    msg.to_string()
                } else if let Some(msg) = map.get("msg").and_then(Value::as_str) {
                    msg.to_string()
                } else if let Some(msg) = map
                    .get("fields")
                    .and_then(|f| f.get("message"))
                    .and_then(Value::as_str)
                {
                    // Handles the Rust `tracing` crate format
                    msg.to_string()
                } else {
                    // If we can't find a message, just use an empty string
                    String::new()
                };

                // --- Extract Level ---
                let level = LogHandler::extract_level(&map);

                // --- Extract Timestamp ---
                let timestamp = map
                    .get("timestamp")
                    .or_else(|| map.get("time"))
                    .or_else(|| map.get("ts"))
                    .and_then(|ts| {
                        if let Some(ts_str) = ts.as_str() {
                            DateTime::parse_from_rfc3339(ts_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .ok()
                        } else if let Some(ts_num) = ts.as_i64() {
                            DateTime::from_timestamp_millis(ts_num)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(now); // Fallback to current time if parsing fails

                LogEntry {
                    timestamp,
                    level,
                    message,
                    raw_payload: log_str,
                }
            }

            // 4. Fallback: Not JSON, or not a JSON Object (e.g., bare array or string)
            _ => {
                return LogEntry {
                    timestamp: now,
                    level: LogLevel::Info,
                    message: log_str.clone(),
                    raw_payload: log_str,
                };
            }
        }
    }

    async fn publish_run_log(&self, run_id: &i64, log_entry: &LogEntry) {
        let channel_name = get_run_logs_channel_name(run_id);
        broadcast_to_channel(
            &self.ws_context,
            &channel_name,
            WsMessage::Data {
                channel: channel_name.clone(),
                payload: serde_json::to_value(log_entry).unwrap(),
            },
        )
        .await;
    }

    pub async fn handle_internal_task_log(&self, run_id: &i64, message: String, level: LogLevel) {
        if message.is_empty() {
            return;
        }
        let log_entry = LogEntry {
            message,
            timestamp: Utc::now(),
            level,
            raw_payload: "".to_string(),
        };
        self.insert_log(*run_id, log_entry.clone()).await;
        self.publish_run_log(run_id, &log_entry).await;
    }

    pub async fn handle_task_run_log(&self, run_id: &i64, log_string: String) {
        if log_string.is_empty() {
            return;
        }
        let log_entry = LogHandler::parse_log(log_string);
        self.insert_log(*run_id, log_entry.clone()).await;
        self.publish_run_log(run_id, &log_entry).await;
    }
}
