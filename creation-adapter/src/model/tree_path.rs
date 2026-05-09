use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct TreePathTable {
    pub world_id: i64,
    pub ancestor_id: i64,
    pub descendant_id: i64,
    pub depth: i32,
}
