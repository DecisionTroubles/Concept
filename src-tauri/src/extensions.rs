use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppError;

fn now_ts() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[taurpc::ipc_type]
pub struct NodeExtensionData {
    pub id: String,
    pub node_id: String,
    pub extension_key: String,
    pub data_json: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn query_node_extension_data(
    conn: &Connection,
    node_id: &str,
    extension_key: Option<&str>,
) -> Result<Vec<NodeExtensionData>, AppError> {
    let sql = if extension_key.is_some() {
        "SELECT id, node_id, extension_key, data_json, created_at, updated_at
         FROM node_extension_data
         WHERE node_id = ?1 AND extension_key = ?2
         ORDER BY updated_at DESC"
    } else {
        "SELECT id, node_id, extension_key, data_json, created_at, updated_at
         FROM node_extension_data
         WHERE node_id = ?1
         ORDER BY updated_at DESC"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(extension_key) = extension_key {
        stmt.query_map(params![node_id, extension_key], |row| {
            Ok(NodeExtensionData {
                id: row.get(0)?,
                node_id: row.get(1)?,
                extension_key: row.get(2)?,
                data_json: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([node_id], |row| {
            Ok(NodeExtensionData {
                id: row.get(0)?,
                node_id: row.get(1)?,
                extension_key: row.get(2)?,
                data_json: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

pub fn upsert_node_extension_data(
    conn: &Connection,
    node_id: &str,
    extension_key: &str,
    data_json: &str,
) -> Result<NodeExtensionData, AppError> {
    let now = now_ts();
    let id = conn
        .query_row(
            "SELECT id FROM node_extension_data WHERE node_id = ?1 AND extension_key = ?2",
            params![node_id, extension_key],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| Uuid::new_v4().to_string());

    conn.execute(
        "INSERT INTO node_extension_data
            (id, node_id, extension_key, data_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(node_id, extension_key) DO UPDATE SET
            data_json = excluded.data_json,
            updated_at = excluded.updated_at",
        params![id, node_id, extension_key, data_json, now],
    )?;

    let rows = query_node_extension_data(conn, node_id, Some(extension_key))?;
    rows.into_iter()
        .next()
        .ok_or_else(|| AppError::Other("Failed to load node extension data after save".to_string()))
}
