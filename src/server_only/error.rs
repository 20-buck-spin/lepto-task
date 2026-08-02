// server_error.rs
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use thiserror::Error;

// Derive Debug always, but only derive Serialize/Deserialize when ssr is enabled
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum AppError {
    #[error("Server fn Error {0}")]
    ServerFnError(ServerFnErrorErr),
    #[error("Dbb error {0}")]
    DbError(String),
    #[error("Task executor error {0}")]
    TaskExecutorError(String),
    #[error("Cron Expression Invalid")]
    CronExprError,
    #[error("Cron Iteration Error")]
    CronIterError,
    #[error("{0}")]
    CustomError(String),
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(value)
    }
}
