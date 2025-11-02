use crate::World;
use typst::WorldExt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WarningRange {
    pub start: usize,
    pub end: usize,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Severity {
    Warning,
    Error,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Diagnostic {
    pub range: Option<WarningRange>,
    pub message: String,
    pub severity: Severity,
}

impl Diagnostic {
    pub(crate) fn from_source_diagnostic(
        world: &World,
        prefix_len: usize,
        source_diag: typst::diag::SourceDiagnostic,
    ) -> Self {
        Self {
            range: world.range(source_diag.span).map(|x| WarningRange {
                start: x.start.saturating_sub(prefix_len),
                end: x.end.saturating_sub(prefix_len),
            }),
            message: source_diag.message.to_string(),
            severity: match source_diag.severity {
                typst::diag::Severity::Error => Severity::Error,
                typst::diag::Severity::Warning => Severity::Warning,
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct CompileResult {
    pub output: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}
