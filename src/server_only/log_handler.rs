use chrono::{DateTime, Utc};
use leptos::logging;
use serde_json::Value;
use sqlx::{query, Pool, Sqlite};

use crate::{
    models::task_models::{LogEntry, LogLevel},
    utils::get_run_logs_channel_name,
    ws::{
        ws_context::{broadcast_to_channel, WsContext},
        WsMessage,
    },
};

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
            let level = extract_level(&map);

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

async fn insert_log(run_id: &i64, log_entry: &LogEntry, db: &Pool<Sqlite>) {
    let level_string: String = log_entry.level.to_string();
    let _ = query!(
        "
            INSERT INTO runlog
            (run_id, created_dt, level, message, raw_payload)
            VALUES
            ($1, $2, $3, $4, $5)
        ",
        run_id,
        log_entry.timestamp,
        level_string,
        log_entry.message,
        log_entry.raw_payload
    )
    .execute(db)
    .await
    .inspect_err(|err| {
        logging::error!(
            "Failed to add run log to db. Log {:?} Error {}",
            log_entry,
            err
        )
    });
}

async fn publish_run_log(ws_context: &WsContext, run_id: &i64, log_entry: &LogEntry) {
    let channel_name = get_run_logs_channel_name(run_id);
    broadcast_to_channel(
        ws_context,
        &channel_name,
        WsMessage::Data {
            channel: channel_name.clone(),
            payload: serde_json::to_value(log_entry).unwrap(),
        },
    )
    .await;
}

pub async fn handle_internal_task_log(
    run_id: &i64,
    message: String,
    level: LogLevel,
    ws_context: &WsContext,
    db: &Pool<Sqlite>,
) {
    if message.is_empty() {
        return;
    }
    let log_entry = LogEntry {
        message,
        timestamp: Utc::now(),
        level,
        raw_payload: "".to_string(),
    };
    insert_log(run_id, &log_entry, db).await;
    publish_run_log(ws_context, run_id, &log_entry).await;
}

pub async fn handle_task_run_log(
    db: &Pool<Sqlite>,
    ws_context: &WsContext,
    run_id: &i64,
    log_string: String,
) {
    if log_string.is_empty() {
        return;
    }
    let log_entry = parse_log(log_string);
    insert_log(run_id, &log_entry, db).await;
    publish_run_log(ws_context, run_id, &log_entry).await;
}
