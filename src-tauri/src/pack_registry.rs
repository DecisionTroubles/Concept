use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::world_registry::{self, WorldPackInfo};

#[derive(Clone)]
struct RegistryPaths {
    registry_path: PathBuf,
    installed_dir: PathBuf,
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
    }
}

fn install_dir_for(source_id: &str) -> Result<PathBuf, AppError> {
    Ok(paths()?.installed_dir.join(source_id))
}

fn pack_file_for(source_id: &str) -> Result<PathBuf, AppError> {
    Ok(install_dir_for(source_id)?.join("pack.json"))
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

fn validate_source_input(input: &GitHubPackSourceInput) -> Result<(), AppError> {
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

fn installed_pack_info(source: &RegistrySource) -> Result<Option<WorldPackInfo>, AppError> {
    let pack_path = pack_file_for(&source.id)?;
    if !pack_path.exists() {
        return Ok(None);
    }
    Ok(Some(world_registry::inspect_pack_file(&pack_path, "installed")))
}

fn entry_from_source(source: &RegistrySource) -> Result<PackRegistryEntry, AppError> {
    let pack_info = installed_pack_info(source)?;
    let install_status = if source.last_error.is_some() && pack_info.is_none() {
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
    };

    Ok(PackRegistryEntry {
        source: to_pack_source(source),
        pack_info,
        install_status: install_status.to_string(),
        last_error: source.last_error.clone(),
    })
}

pub fn get_pack_registry() -> Result<Vec<PackRegistryEntry>, AppError> {
    let manifest = read_manifest()?;
    manifest.sources.iter().map(entry_from_source).collect()
}

pub fn add_github_pack_source(input: GitHubPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_source_input(&input)?;
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
    };
    manifest.sources.push(source.clone());
    manifest.sources.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    write_manifest(&manifest)?;
    entry_from_source(&source)
}

pub fn update_pack_source(id: &str, input: GitHubPackSourceInput) -> Result<PackRegistryEntry, AppError> {
    validate_source_input(&input)?;
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;
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

pub fn remove_pack_source(id: &str) -> Result<(), AppError> {
    let mut manifest = read_manifest()?;
    let previous_len = manifest.sources.len();
    manifest.sources.retain(|source| source.id != id);
    if manifest.sources.len() == previous_len {
        return Err(AppError::NotFound(format!("Pack source '{id}' not found")));
    }
    write_manifest(&manifest)?;
    Ok(())
}

async fn install_source(source: &mut RegistrySource) -> Result<(), AppError> {
    let (pack_json, sha) = fetch_remote_pack(source).await?;
    let install_dir = install_dir_for(&source.id)?;
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

pub async fn install_pack_source(id: &str) -> Result<PackRegistryEntry, AppError> {
    let mut manifest = read_manifest()?;
    let source = manifest
        .sources
        .iter_mut()
        .find(|source| source.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Pack source '{id}' not found")))?;

    match install_source(source).await {
        Ok(()) => {}
        Err(err) => {
            source.last_error = Some(err.to_string());
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

    match fetch_remote_commit_sha(source).await {
        Ok(sha) => {
            source.latest_known_version = Some(sha);
            source.last_checked_at = Some(now_ts());
            source.last_error = None;
        }
        Err(err) => {
            source.last_error = Some(err.to_string());
        }
    }

    let entry = entry_from_source(source)?;
    write_manifest(&manifest)?;
    Ok(entry)
}
