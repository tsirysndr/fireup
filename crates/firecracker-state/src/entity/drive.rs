use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Default, Debug, Clone, Serialize, Deserialize)]
pub struct Drive {
    pub id: String,
    pub name: String,
    pub vm_id: String,
    pub path_on_host: String,
    pub is_root_device: bool,
    pub is_read_only: bool,
}
