use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataWithCount<T> {
    pub data: Vec<T>,
    pub total_count: i64,
}
