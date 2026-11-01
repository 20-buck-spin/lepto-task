use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[cfg(feature = "ssr")]
use sqlx::{prelude::FromRow, Row};

#[cfg_attr(feature = "ssr", derive(FromRow, Debug))]
#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub name: Option<String>,
    pub command: String,
    pub working_dir: String,
    pub cron_expr: String,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RunStatus {
    InProcess,
    Completed,
    Failed,
    Terminated,
    Cancelled,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::InProcess => write!(f, "InProcess"),
            RunStatus::Completed => write!(f, "Completed"),
            RunStatus::Failed => write!(f, "Failed"),
            RunStatus::Terminated => write!(f, "Terminated"),
            RunStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskRunList {
    pub run: TaskRun,
    pub task: Task,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskRun {
    pub id: i64,
    pub task_id: i64,
    pub created_dt: DateTime<Utc>,
    pub finish_dt: Option<DateTime<Utc>>,
    pub status: RunStatus,
    pub max_ram_bytes: u64,
    pub max_cpu_percent: f32,
}

#[cfg(feature = "ssr")]
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for TaskRun {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        Ok(TaskRun {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            created_dt: row.try_get("created_dt")?,
            finish_dt: row.try_get("finish_dt")?,
            max_cpu_percent: row.try_get("max_cpu_percent")?,
            max_ram_bytes: row.try_get("max_ram_bytes")?,
            status: {
                let status_str: String = row.try_get("status")?;
                match status_str.as_str() {
                    "InProcess" => RunStatus::InProcess,
                    "Completed" => RunStatus::Completed,
                    "Failed" => RunStatus::Failed,
                    "Terminated" => RunStatus::Terminated,
                    "Cancelled" => RunStatus::Cancelled,
                    _ => return Err(sqlx::Error::Decode("Invalid status value".into())),
                }
            },
        })
    }
}

#[cfg_attr(feature = "ssr", derive(FromRow, Debug))]
#[derive(Serialize, Deserialize, Clone)]
pub struct RunningTaskPublic {
    pub task: Task,
    pub run_id: i64,
    pub max_cpu_usage: f32,
    pub current_cpu_usage: f32,
    pub max_ram_usage: u64,
    pub current_ram_usage: u64,
    pub started_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskRunDetailsPublic {
    pub task: Task,
    pub run_id: i64,
    pub max_cpu_usage: f32,
    pub current_cpu_usage: f32,
    pub max_ram_usage: u64,
    pub current_ram_usage: u64,
    pub started_at: DateTime<Utc>,
    pub status: RunStatus,
    pub finish_dt: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunLog {
    pub id: i64,
    pub run_id: i64,
    pub created_dt: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub raw_payload: String,
}

#[cfg(feature = "ssr")]
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for RunLog {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(RunLog {
            id: row.try_get("id")?,
            run_id: row.try_get("run_id")?,
            created_dt: row.try_get("created_dt")?,
            level: {
                let level_str: String = row.try_get("level")?;
                match level_str.try_into() {
                    Ok(lvl) => lvl,
                    Err(err) => {
                        return Err(sqlx::Error::Decode(
                            format!("Could not decode log level. Err {}", err).into(),
                        ))
                    }
                }
            },
            message: row.try_get("message")?,
            raw_payload: row.try_get("raw_payload")?,
        })
    }
}

pub fn deserialize_html_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;

    match s {
        Some(val) if !val.is_empty() => {
            // HTML datetime-local yields "YYYY-MM-DDTHH:MM" (length 16)
            // Or "YYYY-MM-DDTHH:MM:SS" if step="1" is used.
            // We can pad seconds and the 'Z' timezone specifier.
            let normalized = if val.ends_with('Z') {
                val
            } else if val.len() == 16 {
                format!("{}:00Z", val) // Missing seconds and timezone
            } else {
                format!("{}Z", val) // Missing timezone only
            };

            normalized
                .parse::<DateTime<Utc>>()
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        _ => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct RunLogFilters {
    pub run_id: i64,
    pub levels: Vec<String>,
    pub query_string: Option<String>,
    #[serde(default, deserialize_with = "deserialize_html_datetime")]
    pub created_dt_from: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_html_datetime")]
    pub created_dt_to: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

impl Default for RunLogFilters {
    fn default() -> Self {
        RunLogFilters {
            limit: 30,
            offset: 0,
            levels: vec![],
            created_dt_from: None,
            created_dt_to: None,
            query_string: None,
            run_id: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TaskFilters {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCreateUpdate {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub command: String,
    pub working_dir: String,
    pub cron_expr: String,
    pub is_active: bool,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct LogEntry {
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub raw_payload: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl TryFrom<&str> for LogLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, String> {
        match value.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" => Ok(LogLevel::Critical),
            _ => Err(format!("Invalid log level: {}", value)),
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        TryFrom::try_from(value.as_str())
    }
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Critical => "CRITICAL",
            LogLevel::Debug => "DEBUG",
            LogLevel::Error => "ERROR",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
