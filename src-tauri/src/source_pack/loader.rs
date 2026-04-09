use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::AppError;

use super::compiler;
use super::diagnostics::{diagnostic, sort_diagnostics, SourcePackDiagnostics, SourcePackProbeResult};
use super::markdown::split_frontmatter;
use super::types::{
    SourceConnectionLayer, SourceGroup, SourceLayer, SourceNode, SourceNodeFrontmatter, SourceNoteType, SourcePack, SourcePackManifest,
    SourceRelationKind, SourceTheme,
};

fn usize_to_u32(value: usize) -> Option<u32> {
    u32::try_from(value).ok()
}

fn read_toml_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let raw = fs::read_to_string(path).map_err(|err| AppError::Other(format!("Failed to read {}: {err}", path.display())))?;
    toml::from_str::<T>(&raw).map_err(|err| AppError::Other(format!("Invalid TOML in {}: {err}", path.display())))
}

fn read_optional_dir_entries<T: serde::de::DeserializeOwned>(root_dir: &Path, relative: &str) -> Result<Vec<T>, AppError> {
    let dir = root_dir.join(relative);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|err| AppError::Other(format!("Failed to read {}: {err}", dir.display())))? {
        let entry = entry.map_err(|err| AppError::Other(format!("Failed to read {}: {err}", dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("toml")).unwrap_or(false) {
            paths.push(path);
        }
    }
    paths.sort();

    paths
        .into_iter()
        .map(|path| read_toml_file::<T>(&path))
        .collect()
}

fn read_nodes(root_dir: &Path) -> Result<Vec<SourceNode>, AppError> {
    let nodes_dir = root_dir.join("nodes");
    if !nodes_dir.exists() {
        return Err(AppError::Other(format!(
            "Source pack is missing nodes directory: {}",
            nodes_dir.display()
        )));
    }

    let mut paths = Vec::new();
    for entry in WalkDir::new(&nodes_dir).min_depth(1).into_iter().filter_map(Result::ok) {
        let path = entry.into_path();
        if path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("md")).unwrap_or(false) {
            paths.push(path);
        }
    }
    paths.sort();

    let mut nodes = Vec::new();
    for path in paths {
        let raw = fs::read_to_string(&path).map_err(|err| AppError::Other(format!("Failed to read {}: {err}", path.display())))?;
        let (frontmatter_raw, body) = split_frontmatter(&raw)?;
        let frontmatter = toml::from_str::<SourceNodeFrontmatter>(&frontmatter_raw)
            .map_err(|err| AppError::Other(format!("Invalid TOML frontmatter in {}: {err}", path.display())))?;
        nodes.push(SourceNode {
            file_path: path,
            frontmatter,
            body,
        });
    }

    Ok(nodes)
}

pub fn resolve_source_pack_root(path: &Path) -> Result<PathBuf, AppError> {
    if path.is_file() {
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        return match file_name {
            "pack.toml" | "pack.json" => path.parent().map(Path::to_path_buf).ok_or_else(|| {
                AppError::Other(format!("Could not resolve pack root for {}", path.display()))
            }),
            _ => Err(AppError::Other(format!(
                "Unsupported pack file '{}'. Expected pack.toml or pack.json.",
                path.display()
            ))),
        };
    }

    if path.join("pack.toml").exists() || path.join("pack.json").exists() {
        return Ok(path.to_path_buf());
    }

    for entry in WalkDir::new(path).max_depth(3).into_iter().filter_map(Result::ok) {
        let candidate = entry.into_path();
        let name = candidate.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        if matches!(name, "pack.toml" | "pack.json") {
            return candidate.parent().map(Path::to_path_buf).ok_or_else(|| {
                AppError::Other(format!("Could not resolve pack root for {}", candidate.display()))
            });
        }
    }

    Err(AppError::Other(format!(
        "No source pack or runtime pack found under {}",
        path.display()
    )))
}

pub fn load_source_pack_from_path(path: &Path) -> Result<SourcePack, AppError> {
    let root_dir = resolve_source_pack_root(path)?;
    let pack_file = root_dir.join("pack.toml");
    if !pack_file.exists() {
        return Err(AppError::Other(format!(
            "Source pack is missing pack.toml under {}",
            root_dir.display()
        )));
    }

    Ok(SourcePack {
        root_dir: root_dir.clone(),
        pack_file: pack_file.clone(),
        manifest: read_toml_file::<SourcePackManifest>(&pack_file)?,
        theme: {
            let path = root_dir.join("theme.toml");
            if path.exists() { Some(read_toml_file::<SourceTheme>(&path)?) } else { None }
        },
        groups: read_optional_dir_entries::<SourceGroup>(&root_dir, "groups")?,
        layers: read_optional_dir_entries::<SourceLayer>(&root_dir, "layers")?,
        connection_layers: read_optional_dir_entries::<SourceConnectionLayer>(&root_dir, "connection-layers")?,
        relation_kinds: read_optional_dir_entries::<SourceRelationKind>(&root_dir, "relation-kinds")?,
        note_types: read_optional_dir_entries::<SourceNoteType>(&root_dir, "note-types")?,
        nodes: read_nodes(&root_dir)?,
    })
}

fn inspect_runtime_pack(path: &Path, input_path: &Path) -> SourcePackProbeResult {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) => {
            return SourcePackProbeResult {
                kind: "invalid".into(),
                input_path: input_path.to_string_lossy().to_string(),
                resolved_path: Some(path.to_string_lossy().to_string()),
                world_id: None,
                world_name: None,
                note_type_count: None,
                node_count: None,
                diagnostics: vec![diagnostic(
                    "error",
                    "runtime_pack_read_failed",
                    format!("Failed to read runtime pack: {err}"),
                    Some(path.to_string_lossy().to_string()),
                    None,
                    None,
                    None,
                )],
            }
        }
    };

    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => SourcePackProbeResult {
            kind: "runtime_pack".into(),
            input_path: input_path.to_string_lossy().to_string(),
            resolved_path: Some(path.to_string_lossy().to_string()),
            world_id: value
                .get("world")
                .and_then(|world| world.get("id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            world_name: value
                .get("world")
                .and_then(|world| world.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            note_type_count: value
                .get("note_types")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| usize_to_u32(items.len())),
            node_count: value
                .get("nodes")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| usize_to_u32(items.len())),
            diagnostics: Vec::new(),
        },
        Err(err) => SourcePackProbeResult {
            kind: "invalid".into(),
            input_path: input_path.to_string_lossy().to_string(),
            resolved_path: Some(path.to_string_lossy().to_string()),
            world_id: None,
            world_name: None,
            note_type_count: None,
            node_count: None,
            diagnostics: vec![diagnostic(
                "error",
                "runtime_pack_invalid_json",
                format!("Invalid runtime pack JSON: {err}"),
                Some(path.to_string_lossy().to_string()),
                None,
                None,
                None,
            )],
        },
    }
}

pub fn probe_source_pack_path(path: &Path) -> Result<SourcePackProbeResult, AppError> {
    let root_dir = resolve_source_pack_root(path)?;
    let source_pack_path = root_dir.join("pack.toml");
    if source_pack_path.exists() {
        let source_pack = load_source_pack_from_path(&root_dir)?;
        let (compiled, diagnostics) = compiler::compile_source_pack(&source_pack)?;
        let (world_id, world_name, note_type_count, node_count) = super::pack_summary(&compiled);
        let mut diagnostics = diagnostics.diagnostics;
        sort_diagnostics(&mut diagnostics);
        return Ok(SourcePackProbeResult {
            kind: if diagnostics.iter().any(|item| item.severity == "error") {
                "invalid".into()
            } else {
                "source_pack".into()
            },
            input_path: path.to_string_lossy().to_string(),
            resolved_path: Some(root_dir.to_string_lossy().to_string()),
            world_id: Some(world_id),
            world_name: Some(world_name),
            note_type_count: Some(note_type_count),
            node_count: Some(node_count),
            diagnostics,
        });
    }

    let runtime_pack_path = root_dir.join("pack.json");
    if runtime_pack_path.exists() {
        return Ok(inspect_runtime_pack(&runtime_pack_path, path));
    }

    let mut diagnostics = SourcePackDiagnostics::new();
    diagnostics.push(diagnostic(
        "error",
        "pack_not_found",
        format!("No pack.toml or pack.json found under {}", root_dir.display()),
        Some(root_dir.to_string_lossy().to_string()),
        None,
        None,
        None,
    ));
    Ok(SourcePackProbeResult {
        kind: "invalid".into(),
        input_path: path.to_string_lossy().to_string(),
        resolved_path: Some(root_dir.to_string_lossy().to_string()),
        world_id: None,
        world_name: None,
        note_type_count: None,
        node_count: None,
        diagnostics: diagnostics.diagnostics,
    })
}
