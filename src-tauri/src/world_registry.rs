use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rusqlite::{params, Connection};
use serde_json::Value;

use crate::domain;
use crate::error::AppError;
use crate::graph;
use crate::source_pack;

#[taurpc::ipc_type]
pub struct CreateLocalWorldInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub template: String,
}

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

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in input.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn local_root() -> Result<PathBuf, AppError> {
    scan_roots()
        .iter()
        .find(|root| root.kind == "local")
        .map(|root| root.path.clone())
        .ok_or_else(|| AppError::Other("Local world root is not configured".into()))
}

fn toml_basic_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn write_source_pack_file(path: &Path, content: &str) -> Result<(), AppError> {
    fs::write(path, content).map_err(|e| AppError::Other(e.to_string()))
}

fn write_local_world_source_pack(
    world_dir: &Path,
    world_id: &str,
    world_name: &str,
    description: Option<&str>,
    template: &str,
) -> Result<(), AppError> {
    let is_starter = template.eq_ignore_ascii_case("starter");
    let description = if is_starter {
        description.unwrap_or("Starter world scaffold for quick Concept authoring.")
    } else {
        description.unwrap_or("Fresh local world scaffold for source-pack authoring.")
    };
    let root_node_id = if is_starter { "welcome" } else { "start-here" };
    let root_title = if is_starter { "Start here" } else { "New world" };
    let root_summary = if is_starter {
        "This starter world is ready for direct node authoring."
    } else {
        "This blank world gives you the smallest valid source-pack scaffold to build from."
    };
    let root_details = if is_starter {
        "Place a node in author mode, then shape links and content from there."
    } else {
        "Rename this node, add nearby concepts, and grow the world from here. The pack files stay readable on disk."
    };
    let root_example = if is_starter {
        "Use the authoring panel to create your next node and connect it."
    } else {
        "Create a second node, connect it with rel-link, then expand note fields when the structure feels right."
    };
    let world_id_toml = toml_basic_string(world_id);
    let world_name_toml = toml_basic_string(world_name);
    let description_toml = toml_basic_string(description);
    let root_node_id_toml = toml_basic_string(root_node_id);

    fs::create_dir_all(world_dir).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("note-types")).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("relation-kinds")).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("layers")).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("connection-layers")).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("groups")).map_err(|e| AppError::Other(e.to_string()))?;
    fs::create_dir_all(world_dir.join("nodes")).map_err(|e| AppError::Other(e.to_string()))?;

    write_source_pack_file(
        &world_dir.join("pack.toml"),
        &format!(
            r#"version = "source-v1"

[world]
id = "{world_id_toml}"
name = "{world_name_toml}"
description = "{description_toml}"
root_node = "{root_node_id_toml}"
default_note_type = "concept-basic"

[authoring]
default_group = "core"
default_layer = "main"

[layout]
mode = "force"
node_spacing = 7.0
group_spacing = 18.0
focus_child_radius = 8.0
allow_explicit_positions = true

[build]
emit_runtime_pack = true
runtime_pack_version = "2"
"#
        ),
    )?;
    write_source_pack_file(
        &world_dir.join("theme.toml"),
        r##"[node_types.concept]
color = "#8fb3ff"
emissive = "#314e93"
radius = 1.0
"##,
    )?;
    write_source_pack_file(
        &world_dir.join("groups").join("core.toml"),
        r##"id = "core"
label = "Core"

[style]
color = "#38bdf8"
emissive = "#155e75"
"##,
    )?;
    write_source_pack_file(
        &world_dir.join("layers").join("main.toml"),
        r#"id = "main"
label = "Main"
"#,
    )?;
    write_source_pack_file(
        &world_dir.join("connection-layers").join("all-links.toml"),
        r##"id = "all-links"
label = "All links"

[style]
color = "#5dd6ff"
width = 2.4
"##,
    )?;
    write_source_pack_file(
        &world_dir.join("relation-kinds").join("rel-link.toml"),
        "id = \"rel-link\"\nlabel = \"Link\"\ndirected = true\ndefault_weight = 1.0\n",
    )?;
    write_source_pack_file(
        &world_dir.join("note-types").join("concept-basic.toml"),
        r#"id = "concept-basic"
name = "Concept Basic"
is_default = true

[[fields]]
key = "Summary"
label = "Summary"
type = "string"
widget = "text"

[[fields]]
key = "Details"
label = "Details"
type = "string"
widget = "markdown"

[[fields]]
key = "Example"
label = "Example"
type = "string"
widget = "long_text"

[[pages]]
id = "overview"
label = "Overview"
kind = "content"
fields = ["Summary", "Details"]

[[pages]]
id = "example"
label = "Example"
kind = "content"
fields = ["Example"]

[[pages]]
id = "connections"
label = "Connections"
kind = "built_in"
source = "connections"
"#,
    )?;
    write_source_pack_file(
        &world_dir.join("nodes").join(format!("{root_node_id}.md")),
        &format!(
            r#"+++
id = "{root_node_id}"
title = "{root_title}"
node_type = "concept"
note_type = "concept-basic"
group = "core"
layer = "main"
tags = ["starter"]

[placement]
x = 0.0
y = 0.0
z = 0.0
locked = true
+++

# Summary
{root_summary}

# Details
{root_details}

# Example
{root_example}
"#
        ),
    )?;

    let compile_result = source_pack::compile_source_pack_json_from_path(world_dir)?;
    if compile_result.diagnostics.iter().any(|item| item.severity == "error") {
        return Err(AppError::Other("Generated source pack did not compile cleanly".into()));
    }
    write_source_pack_file(&world_dir.join("pack.json"), &compile_result.pack_json)?;
    Ok(())
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

fn clear_app_state(conn: &Connection, key: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM app_state WHERE key = ?1", [key])?;
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

    let tracked_local_infos = crate::pack_registry::tracked_local_source_pack_infos()?;
    let tracked_local_world_ids: Vec<String> = tracked_local_infos
        .iter()
        .filter_map(|info| info.world_id.clone())
        .collect();

    infos.retain(|info| {
        if info.source_kind != "local" {
            return true;
        }
        match &info.world_id {
            Some(world_id) => !tracked_local_world_ids.iter().any(|tracked_id| tracked_id == world_id),
            None => true,
        }
    });

    infos.extend(tracked_local_infos);

    infos.sort_by(|a, b| {
        let a_name = a.world_name.as_deref().unwrap_or(&a.pack_path);
        let b_name = b.world_name.as_deref().unwrap_or(&b.pack_path);
        a_name.cmp(b_name).then(a.pack_path.cmp(&b.pack_path))
    });

    Ok(infos)
}

fn find_pack_by_world_id(conn: &Connection, world_id: &str) -> Result<WorldPackInfo, AppError> {
    let mut matches: Vec<WorldPackInfo> = list_world_packs(conn)?
        .into_iter()
        .filter(|info| info.valid && info.world_id.as_deref() == Some(world_id))
        .collect();

    matches.sort_by_key(|info| match info.source_kind.as_str() {
        "local" => 0,
        "installed" => 1,
        _ => 2,
    });

    matches
        .into_iter()
        .next()
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

pub fn delete_local_world(conn: &Connection, pack_path: &str) -> Result<(), AppError> {
    let target = PathBuf::from(pack_path);
    let local_root = scan_roots()
        .iter()
        .find(|root| root.kind == "local")
        .map(|root| root.path.clone())
        .ok_or_else(|| AppError::Other("Local world root is not configured".into()))?;

    if !target.exists() {
        return Err(AppError::NotFound(format!("Local pack '{pack_path}' not found")));
    }

    let canonical_target = fs::canonicalize(&target).map_err(|e| AppError::Other(e.to_string()))?;
    let canonical_local_root = fs::canonicalize(&local_root).map_err(|e| AppError::Other(e.to_string()))?;
    if !canonical_target.starts_with(&canonical_local_root) {
        return Err(AppError::Other("Only managed local world copies can be deleted".into()));
    }

    let info = inspect_pack_file(&canonical_target, "local");
    let active_world_id = get_app_state(conn, "active_world_id")?;
    let loaded_world_id = current_loaded_world_id(conn)?;
    let deleted_world_id = info.world_id.clone();

    let delete_root = canonical_target
        .parent()
        .filter(|parent| parent.starts_with(&canonical_local_root))
        .map(Path::to_path_buf)
        .unwrap_or(canonical_target.clone());

    if delete_root.is_dir() {
        fs::remove_dir_all(&delete_root).map_err(|e| AppError::Other(e.to_string()))?;
    } else {
        fs::remove_file(&delete_root).map_err(|e| AppError::Other(e.to_string()))?;
    }

    if deleted_world_id.as_ref() == active_world_id.as_ref() || deleted_world_id.as_ref() == loaded_world_id.as_ref() {
        graph::reset_data(conn, false)?;
        clear_app_state(conn, "active_world_id")?;
        ensure_active_world_loaded(conn)?;
    }

    Ok(())
}

pub fn create_local_world(conn: &Connection, input: CreateLocalWorldInput) -> Result<WorldPackInfo, AppError> {
    let world_name = input.name.trim();
    if world_name.is_empty() {
        return Err(AppError::Other("World name is required".into()));
    }

    let world_id = input
        .id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(slugify)
        .unwrap_or_else(|| slugify(world_name));
    if world_id.is_empty() {
        return Err(AppError::Other("World id could not be derived from the name".into()));
    }

    if list_world_packs(conn)?
        .iter()
        .any(|info| info.world_id.as_deref() == Some(world_id.as_str()))
    {
        return Err(AppError::Other(format!("World id '{world_id}' already exists")));
    }

    let local_root = local_root()?;
    let world_dir = local_root.join(&world_id);
    if world_dir.exists() {
        return Err(AppError::Other(format!("Local world directory already exists for '{world_id}'")));
    }

    write_local_world_source_pack(
        &world_dir,
        &world_id,
        world_name,
        input.description.as_deref(),
        &input.template,
    )
    .inspect_err(|_| {
        let _ = fs::remove_dir_all(&world_dir);
    })?;

    let pack_path = world_dir.join("pack.json");
    set_app_state(conn, "active_world_id", &world_id)?;

    Ok(inspect_pack_file(&pack_path, "local"))
}
