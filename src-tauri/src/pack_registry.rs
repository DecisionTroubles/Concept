use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::WalkDir;

use crate::anki::{self, AnkiConnectPackSourceInput, AnkiDeckInspectInput, AnkiDeckProbe, AnkiImportConfig, AnkiNoteModelMapping};
use crate::error::AppError;
use crate::world_registry::{self, WorldPackInfo};

#[derive(Clone)]
struct RegistryPaths {
    registry_path: PathBuf,
    installed_dir: PathBuf,
    local_dir: PathBuf,
}

static REGISTRY_PATHS: OnceLock<RegistryPaths> = OnceLock::new();

#[taurpc::ipc_type]
pub struct PackSource {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub repo: String,
    pub path: String,
    pub branch: String,
    pub enabled: bool,
    pub installed_version: Option<String>,
    pub last_checked_at: Option<String>,
    pub last_installed_at: Option<String>,
    pub pinned_ref: Option<String>,
    pub deck_name: Option<String>,
    pub anki_base_url: Option<String>,
    pub grouping_tag_prefix: Option<String>,
    pub include_media: bool,
    pub enforce_own_styles: bool,
    pub note_model_mappings: Option<Vec<AnkiNoteModelMapping>>,
}

#[taurpc::ipc_type]
pub struct PackRegistryEntry {
    pub source: PackSource,
    pub pack_info: Option<WorldPackInfo>,
    pub install_status: String,
    pub last_error: Option<String>,
}

#[taurpc::ipc_type]
pub struct GitHubPackSourceInput {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub path: String,
    pub branch: String,
    pub pinned_ref: Option<String>,
    pub enabled: bool,
}

#[taurpc::ipc_type]
pub struct LocalPackSourceInput {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[taurpc::ipc_type]
pub struct LocalPackPathProbe {
    pub input_path: String,
    pub resolved_pack_path: String,
    pub world_id: Option<String>,
    pub world_name: Option<String>,
    pub suggested_id: String,
    pub suggested_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct RegistrySource {
    id: String,
    name: String,
    provider: String,
    repo: String,
    path: String,
    branch: String,
    enabled: bool,
    installed_version: Option<String>,
    last_checked_at: Option<String>,
    last_installed_at: Option<String>,
    pinned_ref: Option<String>,
    latest_known_version: Option<String>,
    last_error: Option<String>,
    #[serde(default)]
    deck_name: Option<String>,
    #[serde(default)]
    anki_base_url: Option<String>,
    #[serde(default)]
    grouping_tag_prefix: Option<String>,
    #[serde(default)]
    include_media: bool,
    #[serde(default)]
    enforce_own_styles: bool,
    #[serde(default)]
    note_model_mappings: Option<Vec<AnkiNoteModelMapping>>,
}

#[derive(Default, Serialize, Deserialize)]
struct RegistryManifest {
    version: u32,
    #[serde(default)]
    sources: Vec<RegistrySource>,
}

fn now_ts() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn configure(root_dir: PathBuf) -> Result<(), AppError> {
    REGISTRY_PATHS
        .set(RegistryPaths {
            registry_path: root_dir.join("registry.toml"),
            installed_dir: root_dir.join("installed"),
            local_dir: root_dir.join("local"),
        })
        .map_err(|_| AppError::Other("Pack registry already configured".into()))
}

fn paths() -> Result<&'static RegistryPaths, AppError> {
    REGISTRY_PATHS.get().ok_or_else(|| AppError::Other("Pack registry not configured".into()))
}

fn read_manifest() -> Result<RegistryManifest, AppError> {
    let path = &paths()?.registry_path;
    if !path.exists() {
        return Ok(RegistryManifest {
            version: 1,
            sources: Vec::new(),
        });
    }
    let raw = fs::read_to_string(path).map_err(|e| AppError::Other(e.to_string()))?;
    toml::from_str(&raw).map_err(|e| AppError::Other(format!("Invalid registry TOML: {e}")))
}

fn write_manifest(manifest: &RegistryManifest) -> Result<(), AppError> {
    let path = &paths()?.registry_path;
    let raw = toml::to_string_pretty(manifest).map_err(|e| AppError::Other(e.to_string()))?;
    fs::write(path, raw).map_err(|e| AppError::Other(e.to_string()))
}

fn to_pack_source(source: &RegistrySource) -> PackSource {
    PackSource {
        id: source.id.clone(),
        name: source.name.clone(),
        provider: source.provider.clone(),
        repo: source.repo.clone(),
        path: source.path.clone(),
        branch: source.branch.clone(),
        enabled: source.enabled,
        installed_version: source.installed_version.clone(),
        last_checked_at: source.last_checked_at.clone(),
        last_installed_at: source.last_installed_at.clone(),
        pinned_ref: source.pinned_ref.clone(),
        deck_name: source.deck_name.clone(),
        anki_base_url: source.anki_base_url.clone(),
        grouping_tag_prefix: source.grouping_tag_prefix.clone(),
        include_media: source.include_media,
        enforce_own_styles: source.enforce_own_styles,
        note_model_mappings: source.note_model_mappings.clone(),
    }
}

fn managed_dir_for(source: &RegistrySource) -> Result<PathBuf, AppError> {
    let paths = paths()?;
    let root = if matches!(source.provider.as_str(), "local" | "anki-connect") {
        &paths.local_dir
    } else {
        &paths.installed_dir
    };
    Ok(root.join(&source.id))
}

fn managed_pack_file_for(source: &RegistrySource) -> Result<PathBuf, AppError> {
    Ok(managed_dir_for(source)?.join("pack.json"))
}

fn local_source_pack_file(source: &RegistrySource) -> Result<PathBuf, AppError> {
    resolve_local_pack_path(source.path.trim())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_dash = false;
            continue;
        }
        if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn inspect_pack_value(raw: &str) -> Result<(Option<String>, Option<String>), AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|e| AppError::Other(format!("Invalid JSON: {e}")))?;
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
    if version != Some("2") {
        return Err(AppError::Other("Unsupported pack version. Only version \"2\" is supported.".into()));
    }
    if world_id.is_none() || world_name.is_none() {
        return Err(AppError::Other("Pack is missing world.id or world.name.".into()));
    }
    Ok((world_id, world_name))
}

fn probe_pack_file(path: &PathBuf) -> Option<(PathBuf, Option<String>, Option<String>)> {
    let raw = fs::read_to_string(path).ok()?;
    let (world_id, world_name) = inspect_pack_value(&raw).ok()?;
    Some((path.clone(), world_id, world_name))
}

fn resolve_local_pack_probe(input_path: &str) -> Result<LocalPackPathProbe, AppError> {
    let trimmed = input_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Other("Local pack path is required".into()));
    }

    let path = PathBuf::from(trimmed);
    let resolved = if path.is_file() {
        probe_pack_file(&path).ok_or_else(|| AppError::Other(format!("Local pack not found: {}", path.to_string_lossy())))?
    } else if path.is_dir() {
        let direct_pack = path.join("pack.json");
        if let Some(found) = probe_pack_file(&direct_pack) {
            found
        } else {
            let mut fallback: Option<(PathBuf, Option<String>, Option<String>)> = None;
            for entry in WalkDir::new(&path).max_depth(3).into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_file() {
                    continue;
                }
                let candidate = entry.into_path();
                let is_json = candidate
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json"))
                    .unwrap_or(false);
                if !is_json {
                    continue;
                }
                if let Some(found) = probe_pack_file(&candidate) {
                    let is_pack_json = candidate
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.eq_ignore_ascii_case("pack.json"))
                        .unwrap_or(false);
                    if is_pack_json {
                        fallback = Some(found);
                        break;
                    }
                    if fallback.is_none() {
                        fallback = Some(found);
                    }
                }
            }
            fallback.ok_or_else(|| AppError::Other(format!("No valid pack found under: {}", path.to_string_lossy())))?
        }
    } else {
        return Err(AppError::Other(format!("Local pack not found: {}", path.to_string_lossy())));
    };

    let suggested_id = resolved
        .1
        .as_deref()
        .map(slugify)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            slugify(
                resolved
                    .0
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("local-pack"),
            )
        });
    let suggested_name = resolved
        .2
        .clone()
        .unwrap_or_else(|| {
            resolved
                .0
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Local Pack")
                .to_string()
        });

    Ok(LocalPackPathProbe {
        input_path: trimmed.to_string(),
        resolved_pack_path: resolved.0.to_string_lossy().to_string(),
        world_id: resolved.1,
        world_name: resolved.2,
        suggested_id,
        suggested_name,
    })
}

fn resolve_local_pack_path(input_path: &str) -> Result<PathBuf, AppError> {
    Ok(PathBuf::from(resolve_local_pack_probe(input_path)?.resolved_pack_path))
}

pub fn inspect_local_pack_path(input_path: &str) -> Result<LocalPackPathProbe, AppError> {
    resolve_local_pack_probe(input_path)
}

fn source_ref(source: &RegistrySource) -> &str {
    source
        .pinned_ref
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(source.branch.as_str())
}

fn normalize_relative_path(value: &str) -> String {
    value.trim().trim_matches('/').replace('\\', "/")
}

fn raw_pack_url(source: &RegistrySource) -> String {
    let path = normalize_relative_path(&source.path);
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/pack.json",
        source.repo,
        source_ref(source),
        path
    )
}

fn commit_api_url(source: &RegistrySource) -> String {
    let path = normalize_relative_path(&source.path);
    format!(
        "https://api.github.com/repos/{}/commits/{}?path={}",
        source.repo,
        source_ref(source),
        path
    )
}

fn http_client() -> Result<reqwest::Client, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("concept-pack-registry"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| AppError::Other(e.to_string()))
}

async fn fetch_remote_commit_sha(source: &RegistrySource) -> Result<String, AppError> {
    let value = http_client()?
        .get(commit_api_url(source))
        .send()
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
        .error_for_status()
        .map_err(|e| AppError::Other(e.to_string()))?
        .json::<Value>()
        .await
        .map_err(|e| AppError::Other(e.to_string()))?;

    value
        .get("sha")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::Other("GitHub response missing commit sha".into()))
}

async fn fetch_remote_pack(source: &RegistrySource) -> Result<(String, String), AppError> {
    let client = http_client()?;
    let sha = fetch_remote_commit_sha(source).await?;
    let pack_json = client
        .get(raw_pack_url(source))
        .send()
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
        .error_for_status()
        .map_err(|e| AppError::Other(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok((pack_json, sha))
}

fn validate_github_source_input(input: &GitHubPackSourceInput) -> Result<(), AppError> {
    if input.id.trim().is_empty() {
        return Err(AppError::Other("Source id is required".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Other("Source name is required".into()));
    }
    if !input.repo.contains('/') {
        return Err(AppError::Other("GitHub repo must be in owner/repo form".into()));
    }
    if input.path.trim().is_empty() {
        return Err(AppError::Other("Pack folder path is required".into()));
    }
    if input.branch.trim().is_empty() && input.pinned_ref.as_deref().unwrap_or("").trim().is_empty() {
        return Err(AppError::Other("Branch or pinned ref is required".into()));
    }
    Ok(())
}

fn validate_local_source_input(input: &LocalPackSourceInput) -> Result<(), AppError> {
    if input.id.trim().is_empty() {
        return Err(AppError::Other("Source id is required".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Other("Source name is required".into()));
    }
    if input.path.trim().is_empty() {
        return Err(AppError::Other("Local pack path is required".into()));
    }
    Ok(())
}

fn validate_anki_source_input(input: &AnkiConnectPackSourceInput) -> Result<(), AppError> {
    if input.id.trim().is_empty() {
        return Err(AppError::Other("Source id is required".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Other("Source name is required".into()));
    }
    if input.deck_name.trim().is_empty() {
        return Err(AppError::Other("Anki deck name is required".into()));
    }
    if input.grouping_tag_prefix.trim().is_empty() {
        return Err(AppError::Other("Grouping tag prefix is required".into()));
    }
    Ok(())
}

fn managed_pack_info(source: &RegistrySource) -> Result<Option<WorldPackInfo>, AppError> {
    let pack_path = managed_pack_file_for(source)?;
    if !pack_path.exists() {
        return Ok(None);
    }
    let source_kind = if matches!(source.provider.as_str(), "local" | "anki-connect") {
        "local"
    } else {
        "installed"
    };
    Ok(Some(world_registry::inspect_pack_file(&pack_path, source_kind)))
}

fn local_source_pack_info(source: &RegistrySource) -> Result<Option<WorldPackInfo>, AppError> {
    let pack_path = local_source_pack_file(source)?;
    if !pack_path.exists() {
        return Ok(None);
    }
    Ok(Some(world_registry::inspect_pack_file(&pack_path, "local")))
}

fn entry_from_source(source: &RegistrySource) -> Result<PackRegistryEntry, AppError> {
    let pack_info = if source.provider == "local" {
        local_source_pack_info(source)?
    } else {
        managed_pack_info(source)?
    };
    let install_status = if source.provider == "local" {
        if source.last_error.is_some() && pack_info.is_none() {
            "error"
        } else if let Some(info) = &pack_info {
            if !info.valid {
                "invalid"
            } else {
                "tracked_local"
            }
        } else {
            "not_synced"
        }
    } else if source.provider == "anki-connect" {
        if source.last_error.is_some() && pack_info.is_none() {
            "error"
        } else if let Some(info) = &pack_info {
            if !info.valid {
                "invalid"
            } else {
                "tracked_anki"
            }
        } else {
            "not_synced"
        }
    } else {
        if source.last_error.is_some() && pack_info.is_none() {
            "error"
        } else if let Some(info) = &pack_info {
            if !info.valid {
                "invalid"
            } else if source
                .latest_known_version
                .as_ref()
                .zip(source.installed_version.as_ref())
                .map(|(latest, installed)| latest != installed)
                .unwrap_or(false)
            {
                "update_available"
            } else {
                "installed"
            }
        } else {
            "not_installed"
        }
    };

    Ok(PackRegistryEntry {
        source: to_pack_source(source),
        pack_info,
        install_status: install_status.to_string(),
        last_error: source.last_error.clone(),
    })
}

pub fn tracked_local_source_pack_infos() -> Result<Vec<WorldPackInfo>, AppError> {
    let manifest = read_manifest()?;
    let mut infos = Vec::new();

    for source in manifest.sources.iter().filter(|source| source.provider == "local" && source.enabled) {
        match local_source_pack_info(source) {
            Ok(Some(info)) => infos.push(info),
            Ok(None) => {
                infos.push(WorldPackInfo {
                    world_id: None,
                    world_name: None,
                    pack_path: source.path.clone(),
                    source_kind: "local".into(),
                    valid: false,
                    is_active: false,
                    is_loaded: false,
                    error: Some(format!("Local pack not found: {}", source.path)),
                });
            }
            Err(err) => {
                infos.push(WorldPackInfo {
                    world_id: None,
                    world_name: None,
                    pack_path: source.path.clone(),
                    source_kind: "local".into(),
                    valid: false,
                    is_active: false,
                    is_loaded: false,
                    error: Some(err.to_string()),
                });
            }
        }
    }

    Ok(infos)
}

pub fn get_pack_registry() -> Result<Vec<PackRegistryEntry>, AppError> {
    let manifest = read_manifest()?;
    manifest.sources.iter().map(entry_from_source).collect()
}

pub async fn list_anki_decks(base_url: Option<&str>) -> Result<Vec<String>, AppError> {
    anki::list_decks(base_url).await
}

pub async fn inspect_anki_deck(input: AnkiDeckInspectInput) -> Result<AnkiDeckProbe, AppError> {
    anki::inspect_deck(&input).await
}

pub fn add_github_pack_source(input: GitHubPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_github_source_input(&input)?;
    let mut manifest = read_manifest()?;
    if manifest.sources.iter().any(|source| source.id == input.id) {
        return Err(AppError::Other(format!("Pack source '{}' already exists", input.id)));
    }
    let source = RegistrySource {
        id: input.id,
        name: input.name,
        provider: "github".into(),
        repo: input.repo,
        path: input.path,
        branch: input.branch,
        enabled: input.enabled,
        installed_version: None,
        last_checked_at: None,
        last_installed_at: None,
        pinned_ref: input.pinned_ref,
        latest_known_version: None,
        last_error: None,
        deck_name: None,
        anki_base_url: None,
        grouping_tag_prefix: None,
        include_media: false,
        enforce_own_styles: false,
        note_model_mappings: None,
    };
    manifest.sources.push(source.clone());
    manifest.sources.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    write_manifest(&manifest)?;
    entry_from_source(&source)
}

pub fn add_local_pack_source(input: LocalPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_local_source_input(&input)?;
    let mut manifest = read_manifest()?;
    if manifest.sources.iter().any(|source| source.id == input.id) {
        return Err(AppError::Other(format!("Pack source '{}' already exists", input.id)));
    }
    let source = RegistrySource {
        id: input.id,
        name: input.name,
        provider: "local".into(),
        repo: String::new(),
        path: input.path,
        branch: String::new(),
        enabled: input.enabled,
        installed_version: None,
        last_checked_at: None,
        last_installed_at: None,
        pinned_ref: None,
        latest_known_version: None,
        last_error: None,
        deck_name: None,
        anki_base_url: None,
        grouping_tag_prefix: None,
        include_media: false,
        enforce_own_styles: false,
        note_model_mappings: None,
    };
    manifest.sources.push(source.clone());
    manifest.sources.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    write_manifest(&manifest)?;
    entry_from_source(&source)
}

pub fn add_anki_pack_source(input: AnkiConnectPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_anki_source_input(&input)?;
    let mut manifest = read_manifest()?;
    if manifest.sources.iter().any(|source| source.id == input.id) {
        return Err(AppError::Other(format!("Pack source '{}' already exists", input.id)));
    }
    let source = RegistrySource {
        id: input.id,
        name: input.name,
        provider: "anki-connect".into(),
        repo: String::new(),
        path: String::new(),
        branch: String::new(),
        enabled: input.enabled,
        installed_version: None,
        last_checked_at: None,
        last_installed_at: None,
        pinned_ref: None,
        latest_known_version: None,
        last_error: None,
        deck_name: Some(input.deck_name),
        anki_base_url: input.anki_base_url.filter(|value| !value.trim().is_empty()),
        grouping_tag_prefix: Some(input.grouping_tag_prefix),
        include_media: input.include_media,
        enforce_own_styles: input.enforce_own_styles,
        note_model_mappings: input.note_model_mappings,
    };
    manifest.sources.push(source.clone());
    manifest.sources.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    write_manifest(&manifest)?;
    entry_from_source(&source)
}

pub fn update_pack_source(id: &str, input: GitHubPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_github_source_input(&input)?;
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;
    if source.provider != "github" {
        return Err(AppError::Other(format!("Pack source '{id}' is not a GitHub source")));
    }
    source.name = input.name;
    source.repo = input.repo;
    source.path = input.path;
    source.branch = input.branch;
    source.enabled = input.enabled;
    source.pinned_ref = input.pinned_ref;
    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}

pub fn update_local_pack_source(id: &str, input: LocalPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_local_source_input(&input)?;
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;
    if source.provider != "local" {
        return Err(AppError::Other(format!("Pack source '{id}' is not a local source")));
    }
    source.name = input.name;
    source.path = input.path;
    source.enabled = input.enabled;
    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}

pub fn update_anki_pack_source(id: &str, input: AnkiConnectPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_anki_source_input(&input)?;
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;
    if source.provider != "anki-connect" {
        return Err(AppError::Other(format!("Pack source '{id}' is not an Anki source")));
    }
    source.name = input.name;
    source.enabled = input.enabled;
    source.deck_name = Some(input.deck_name);
    source.anki_base_url = input.anki_base_url.filter(|value| !value.trim().is_empty());
    source.grouping_tag_prefix = Some(input.grouping_tag_prefix);
    source.include_media = input.include_media;
    source.enforce_own_styles = input.enforce_own_styles;
    source.note_model_mappings = input.note_model_mappings;
    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}

pub fn remove_pack_source(id: &str) -> Result<(), AppError> {
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter()
        .find(|source| source.id == id)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;
    manifest.sources.retain(|entry| entry.id != id);

    let managed_dir = managed_dir_for(&source)?;
    if managed_dir.exists() {
        fs::remove_dir_all(&managed_dir).map_err(|e| AppError::Other(e.to_string()))?;
    }

    if manifest.sources.iter().any(|entry| entry.id == id) {
        return Err(AppError::NotFound(format!("Pack source '{id}' not found")));
    }
    write_manifest(&manifest)?;
    Ok(())
}

async fn install_source(source: &mut RegistrySource) -> Result<(), AppError> {
    let (pack_json, sha) = fetch_remote_pack(source).await?;
    let install_dir = managed_dir_for(source)?;
    fs::create_dir_all(&install_dir).map_err(|e| AppError::Other(e.to_string()))?;
    let pack_path = install_dir.join("pack.json");
    fs::write(&pack_path, pack_json).map_err(|e| AppError::Other(e.to_string()))?;
    let info = world_registry::inspect_pack_file(&pack_path, "installed");
    if !info.valid {
        return Err(AppError::Other(info.error.unwrap_or_else(|| "Installed pack is invalid".into())));
    }
    let now = now_ts();
    source.installed_version = Some(sha.clone());
    source.latest_known_version = Some(sha);
    source.last_checked_at = Some(now.clone());
    source.last_installed_at = Some(now);
    source.last_error = None;
    Ok(())
}

fn sync_local_source(source: &mut RegistrySource) -> Result<(), AppError> {
    let source_pack_path = local_source_pack_file(source)?;
    if !source_pack_path.exists() {
        return Err(AppError::Other(format!(
            "Local pack not found: {}",
            source_pack_path.to_string_lossy()
        )));
    }

    let pack_json = fs::read_to_string(&source_pack_path).map_err(|e| AppError::Other(e.to_string()))?;
    inspect_pack_value(&pack_json)?;

    let local_dir = managed_dir_for(source)?;
    fs::create_dir_all(&local_dir).map_err(|e| AppError::Other(e.to_string()))?;
    let pack_path = local_dir.join("pack.json");
    fs::write(&pack_path, pack_json).map_err(|e| AppError::Other(e.to_string()))?;
    let info = world_registry::inspect_pack_file(&pack_path, "local");
    if !info.valid {
        return Err(AppError::Other(info.error.unwrap_or_else(|| "Synced local pack is invalid".into())));
    }
    let now = now_ts();
    source.last_checked_at = Some(now.clone());
    source.last_installed_at = Some(now);
    source.last_error = None;
    Ok(())
}

async fn sync_anki_source(source: &mut RegistrySource) -> Result<(), AppError> {
    let deck_name = source
        .deck_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Other("Anki source is missing deck_name".into()))?;
    let grouping_tag_prefix = source
        .grouping_tag_prefix
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "group:".into());
    let anki_base_url = source
        .anki_base_url
        .clone()
        .unwrap_or_else(|| "http://127.0.0.1:8765".into());
    let pack_json = anki::generate_pack_json(&AnkiImportConfig {
        source_id: source.id.clone(),
        source_name: source.name.clone(),
        deck_name,
        anki_base_url,
        grouping_tag_prefix,
        include_media: source.include_media,
        enforce_own_styles: source.enforce_own_styles,
        note_model_mappings: source.note_model_mappings.clone().unwrap_or_default(),
    })
    .await?;

    let local_dir = managed_dir_for(source)?;
    fs::create_dir_all(&local_dir).map_err(|e| AppError::Other(e.to_string()))?;
    let pack_path = local_dir.join("pack.json");
    fs::write(&pack_path, pack_json).map_err(|e| AppError::Other(e.to_string()))?;
    let info = world_registry::inspect_pack_file(&pack_path, "local");
    if !info.valid {
        return Err(AppError::Other(info.error.unwrap_or_else(|| "Generated Anki pack is invalid".into())));
    }
    let now = now_ts();
    source.last_checked_at = Some(now.clone());
    source.last_installed_at = Some(now);
    source.last_error = None;
    Ok(())
}

pub async fn install_pack_source(id: &str) -> Result<PackRegistryEntry, AppError> {
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;

    match source.provider.as_str() {
        "github" => match install_source(source).await {
            Ok(()) => {}
            Err(err) => {
                source.last_error = Some(err.to_string());
            }
        },
        "local" => match sync_local_source(source) {
            Ok(()) => {}
            Err(err) => {
                source.last_error = Some(err.to_string());
            }
        },
        "anki-connect" => match sync_anki_source(source).await {
            Ok(()) => {}
            Err(err) => {
                source.last_error = Some(err.to_string());
            }
        },
        other => {
            source.last_error = Some(format!("Unsupported provider '{other}'"));
        }
    }
    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}

pub async fn refresh_pack_source(id: &str) -> Result<PackRegistryEntry, AppError> {
    install_pack_source(id).await
}

pub async fn check_pack_source_updates(id: &str) -> Result<PackRegistryEntry, AppError> {
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;

    match source.provider.as_str() {
        "github" => match fetch_remote_commit_sha(source).await {
            Ok(sha) => {
                source.latest_known_version = Some(sha);
                source.last_checked_at = Some(now_ts());
                source.last_error = None;
            }
            Err(err) => {
                source.last_error = Some(err.to_string());
            }
        },
        "local" => {
            let source_pack_path = local_source_pack_file(source)?;
            source.last_checked_at = Some(now_ts());
            source.last_error = if source_pack_path.exists() {
                None
            } else {
                Some(format!("Local pack not found: {}", source_pack_path.to_string_lossy()))
            };
        }
        "anki-connect" => {
            source.last_checked_at = Some(now_ts());
            source.last_error = None;
        }
        other => {
            source.last_error = Some(format!("Unsupported provider '{other}'"));
        }
    }

    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}
