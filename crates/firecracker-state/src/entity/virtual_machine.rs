use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachine {
    pub id: String,
    pub name: String,
    pub status: String,
    pub vcpu: u16,
    pub memory: u16,
    pub distro: String,
    pub pid: Option<u32>,
    pub mac_address: String,
    pub bridge: String,
    pub tap: String,
    pub api_socket: String,
    pub project_dir: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}
