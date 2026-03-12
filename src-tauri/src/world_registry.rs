use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rusqlite::{params, Connection};
use serde_json::Value;

use crate::domain;
use crate::error::AppError;
use crate::graph;

#[derive(Clone)]
pub struct ScanRoot {
    pub kind: String,
    pub path: PathBuf,
}

static SCAN_ROOTS: OnceLock<Vec<ScanRoot>> = OnceLock::new();

#[taurpc::ipc_type]
pub struct WorldPackInfo {
    pub world_id: Option<String>,
    pub world_name: Option<String>,
    pub pack_path: String,
    pub source_kind: String,
    pub valid: bool,
    pub is_active: bool,
    pub is_loaded: bool,
    pub error: Option<String>,
}

fn now_ts() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn configure_scan_roots(roots: Vec<ScanRoot>) -> Result<(), AppError> {
    SCAN_ROOTS
        .set(roots)
        .map_err(|_| AppError::Other("World scan roots already configured".into()))
}

fn scan_roots() -> &'static [ScanRoot] {
    SCAN_ROOTS.get().map(Vec::as_slice).unwrap_or(&[])
}

fn get_app_state(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let value = conn.query_row(
        "SELECT value FROM app_state WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    );
    match value {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(AppError::Database(err)),
    }
}

fn set_app_state(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    let updated_at = now_ts();
    conn.execute(
        "INSERT INTO app_state (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at",
        params![key, value, updated_at],
    )?;
    Ok(())
}

fn current_loaded_world_id(conn: &Connection) -> Result<Option<String>, AppError> {
    let value = conn.query_row(
        "SELECT id FROM worlds ORDER BY created_at DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match value {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(AppError::Database(err)),
    }
}

fn collect_pack_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), AppError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|e| AppError::Other(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Other(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_pack_files(&path, out)?;
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("pack.json"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
    Ok(())
}

pub fn inspect_pack_file(path: &Path, source_kind: &str) -> WorldPackInfo {
    let pack_path = path.to_string_lossy().to_string();
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) => {
            return WorldPackInfo {
                world_id: None,
                world_name: None,
                pack_path,
                source_kind: source_kind.to_string(),
                valid: false,
                is_active: false,
                is_loaded: false,
                error: Some(format!("Failed to read pack: {err}")),
            }
        }
    };

    let value: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => {
            return WorldPackInfo {
                world_id: None,
                world_name: None,
                pack_path,
                source_kind: source_kind.to_string(),
                valid: false,
                is_active: false,
                is_loaded: false,
                error: Some(format!("Invalid JSON: {err}")),
            }
        }
    };

    let version = value.get("version").and_then(Value::as_str);
    let world_id = value
        .get("world")
        .and_then(|world| world.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let world_name = value
        .get("world")
        .and_then(|world| world.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let error = if version != Some("2") {
        Some("Unsupported pack version. Only version \"2\" is supported.".to_string())
    } else if world_id.is_none() || world_name.is_none() {
        Some("Pack is missing world.id or world.name.".to_string())
    } else {
        None
    };

    WorldPackInfo {
        world_id,
        world_name,
        pack_path,
        source_kind: source_kind.to_string(),
        valid: error.is_none(),
        is_active: false,
        is_loaded: false,
        error,
    }
}

pub fn list_world_packs(conn: &Connection) -> Result<Vec<WorldPackInfo>, AppError> {
    let active_world_id = get_app_state(conn, "active_world_id")?;
    let loaded_world_id = current_loaded_world_id(conn)?;

    let mut infos = Vec::new();
    for root in scan_roots() {
        let mut files = Vec::new();
        collect_pack_files(&root.path, &mut files)?;
        files.sort();
        for file in files {
            let mut info = inspect_pack_file(&file, &root.kind);
            if let Some(world_id) = &info.world_id {
                info.is_active = active_world_id.as_ref() == Some(world_id);
                info.is_loaded = loaded_world_id.as_ref() == Some(world_id);
            }
            infos.push(info);
        }
    }

    infos.sort_by(|a, b| {
        let a_name = a.world_name.as_deref().unwrap_or(&a.pack_path);
        let b_name = b.world_name.as_deref().unwrap_or(&b.pack_path);
        a_name.cmp(b_name).then(a.pack_path.cmp(&b.pack_path))
    });

    Ok(infos)
}

fn find_pack_by_world_id(conn: &Connection, world_id: &str) -> Result<WorldPackInfo, AppError> {
    list_world_packs(conn)?
        .into_iter()
        .find(|info| info.valid && info.world_id.as_deref() == Some(world_id))
        .ok_or_else(|| AppError::NotFound(format!("World pack '{world_id}' not found")))
}

fn load_pack_file(conn: &Connection, world_id: &str) -> Result<(), AppError> {
    let selected = find_pack_by_world_id(conn, world_id)?;
    let json = fs::read_to_string(&selected.pack_path).map_err(|e| AppError::Other(e.to_string()))?;
    graph::reset_data(conn, false)?;
    graph::ensure_default_note_types(conn)?;
    domain::seed_pack(conn, &json)?;
    graph::reconcile_duplicate_layers(conn)?;
    set_app_state(conn, "active_world_id", world_id)?;
    Ok(())
}

pub fn ensure_active_world_loaded(conn: &Connection) -> Result<(), AppError> {
    if current_loaded_world_id(conn)?.is_some() {
        return Ok(());
    }

    let packs = list_world_packs(conn)?;
    let selected_world_id = get_app_state(conn, "active_world_id")?;
    let selected = selected_world_id
        .and_then(|world_id| {
            packs.iter()
                .find(|info| info.valid && info.world_id.as_deref() == Some(world_id.as_str()))
                .and_then(|info| info.world_id.clone())
        })
        .or_else(|| packs.iter().find(|info| info.valid).and_then(|info| info.world_id.clone()));

    let Some(world_id) = selected else {
        return Ok(());
    };

    load_pack_file(conn, &world_id)
}

pub fn select_world(conn: &Connection, world_id: &str) -> Result<(), AppError> {
    load_pack_file(conn, world_id)
}

pub fn reload_active_world(conn: &Connection) -> Result<(), AppError> {
    let active_world_id = get_app_state(conn, "active_world_id")?
        .or_else(|| current_loaded_world_id(conn).ok().flatten())
        .ok_or_else(|| AppError::Other("No active world selected".into()))?;
    load_pack_file(conn, &active_world_id)
}
