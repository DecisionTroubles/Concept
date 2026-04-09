#[taurpc::ipc_type]
pub struct SourcePackDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub entity_id: Option<String>,
}

#[taurpc::ipc_type]
pub struct SourcePackDiagnostics {
    pub diagnostics: Vec<SourcePackDiagnostic>,
}

impl SourcePackDiagnostics {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    pub fn push(&mut self, diagnostic: SourcePackDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.severity == "error")
    }
}

#[taurpc::ipc_type]
pub struct SourcePackProbeResult {
    pub kind: String,
    pub input_path: String,
    pub resolved_path: Option<String>,
    pub world_id: Option<String>,
    pub world_name: Option<String>,
    pub note_type_count: Option<u32>,
    pub node_count: Option<u32>,
    pub diagnostics: Vec<SourcePackDiagnostic>,
}

#[taurpc::ipc_type]
pub struct SourcePackCompileResult {
    pub pack_json: String,
    pub diagnostics: Vec<SourcePackDiagnostic>,
    pub world_id: String,
    pub world_name: String,
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    }
}

pub fn sort_diagnostics(diagnostics: &mut [SourcePackDiagnostic]) {
    diagnostics.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then(a.file.cmp(&b.file))
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
            .then(a.code.cmp(&b.code))
    });
}

pub fn diagnostic(
    severity: &str,
    code: impl Into<String>,
    message: impl Into<String>,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
    entity_id: Option<String>,
) -> SourcePackDiagnostic {
    SourcePackDiagnostic {
        severity: severity.to_string(),
        code: code.into(),
        message: message.into(),
        file,
        line,
        column,
        entity_id,
    }
}
