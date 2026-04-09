pub mod compiler;
pub mod diagnostics;
pub mod loader;
pub mod markdown;
pub mod note_types;
pub mod types;

use std::path::Path;

use serde_json::Value;

use crate::domain::DomainPackV2;
use crate::error::AppError;
pub use diagnostics::{SourcePackCompileResult, SourcePackDiagnostic, SourcePackDiagnostics, SourcePackProbeResult};
pub use types::SourcePack;

pub fn probe_source_pack_path(path: &Path) -> Result<SourcePackProbeResult, AppError> {
    loader::probe_source_pack_path(path)
}

pub fn load_source_pack_from_path(path: &Path) -> Result<SourcePack, AppError> {
    loader::load_source_pack_from_path(path)
}

pub fn compile_source_pack_from_path(path: &Path) -> Result<(DomainPackV2, SourcePackDiagnostics), AppError> {
    let source_pack = load_source_pack_from_path(path)?;
    compiler::compile_source_pack(&source_pack)
}

pub fn compile_source_pack_json_from_path(path: &Path) -> Result<SourcePackCompileResult, AppError> {
    let (pack, diagnostics) = compile_source_pack_from_path(path)?;
    let pack_json = serde_json::to_string_pretty(&pack)
        .map_err(|err| AppError::Other(format!("Failed to serialize runtime pack JSON: {err}")))?;

    Ok(SourcePackCompileResult {
        pack_json,
        diagnostics: diagnostics.diagnostics,
        world_id: pack.world.id,
        world_name: pack.world.name,
    })
}

pub fn validate_source_pack_from_path(path: &Path) -> Result<SourcePackDiagnostics, AppError> {
    let source_pack = load_source_pack_from_path(path)?;
    let (_, diagnostics) = compiler::compile_source_pack(&source_pack)?;
    Ok(diagnostics)
}

pub fn pack_summary(pack: &DomainPackV2) -> (String, String, u32, u32) {
    (
        pack.world.id.clone(),
        pack.world.name.clone(),
        u32::try_from(pack.note_types.len()).unwrap_or(u32::MAX),
        u32::try_from(pack.nodes.len()).unwrap_or(u32::MAX),
    )
}

pub fn toml_value_to_json(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(v) => Value::String(v.clone()),
        toml::Value::Integer(v) => Value::Number((*v).into()),
        toml::Value::Float(v) => serde_json::Number::from_f64(*v)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Datetime(v) => Value::String(v.to_string()),
        toml::Value::Array(values) => Value::Array(values.iter().map(toml_value_to_json).collect()),
        toml::Value::Table(table) => Value::Object(
            table
                .iter()
                .map(|(key, value)| (key.clone(), toml_value_to_json(value)))
                .collect(),
        ),
    }
}
