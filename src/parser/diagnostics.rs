/// High-level diagnostic categories surfaced to users.
#[derive(Debug)]
pub enum DiagnosticKind {
    Unsupported,
    InvalidSpec,
    IncompleteData,
}

impl DiagnosticKind {
    /// Returns a stable machine-readable label for the diagnostic category.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::Unsupported => "unsupported",
            DiagnosticKind::InvalidSpec => "invalid_spec",
            DiagnosticKind::IncompleteData => "incomplete_data",
        }
    }
}

/// Single parser diagnostic with endpoint context and human-readable detail.
#[derive(Debug)]
pub struct ParseIssue {
    pub kind: DiagnosticKind,
    pub code: Option<&'static str>,
    pub stage: &'static str,
    pub path: Option<String>,
    pub method: Option<String>,
    pub status: Option<String>,
    pub detail: String,
}

impl ParseIssue {
    /// Convenience accessor for the issue kind label.
    pub fn kind_str(&self) -> &'static str {
        self.kind.as_str()
    }
}

/// Parsing context propagated through nested parsing calls.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ParseCtx<'a> {
    pub(crate) path: Option<&'a str>,
    pub(crate) method: Option<&'a str>,
    pub(crate) status: Option<&'a str>,
}

impl<'a> ParseCtx<'a> {
    /// Creates a parsing context with optional path/method/status coordinates.
    pub(crate) fn new(
        path: Option<&'a str>,
        method: Option<&'a str>,
        status: Option<&'a str>,
    ) -> Self {
        Self {
            path,
            method,
            status,
        }
    }

    /// Returns a copy of the context with an updated status code.
    pub(crate) fn with_status(self, status: Option<&'a str>) -> Self {
        Self { status, ..self }
    }
}

/// Maps internal stage identifiers to external diagnostic categories.
fn diagnostic_kind_from_stage(stage: &'static str) -> DiagnosticKind {
    match stage {
        "schema" => DiagnosticKind::Unsupported,
        "schema.ref" => DiagnosticKind::InvalidSpec,
        "parameters" => DiagnosticKind::IncompleteData,
        "parameters.ref" => DiagnosticKind::InvalidSpec,
        "response.status" | "response.ref" => DiagnosticKind::InvalidSpec,
        "response" => DiagnosticKind::IncompleteData,
        "path_methods" => DiagnosticKind::InvalidSpec,
        _ => DiagnosticKind::InvalidSpec,
    }
}

/// Appends a diagnostic entry to the shared diagnostics list.
pub(crate) fn issue(
    issues: &mut Vec<ParseIssue>,
    stage: &'static str,
    ctx: ParseCtx<'_>,
    detail: impl Into<String>,
) {
    issue_with_code(issues, stage, None, ctx, detail);
}

/// Appends a diagnostic entry with a stable machine-readable code.
pub(crate) fn issue_with_code(
    issues: &mut Vec<ParseIssue>,
    stage: &'static str,
    code: Option<&'static str>,
    ctx: ParseCtx<'_>,
    detail: impl Into<String>,
) {
    issues.push(ParseIssue {
        kind: diagnostic_kind_from_stage(stage),
        code,
        stage,
        path: ctx.path.map(String::from),
        method: ctx.method.map(String::from),
        status: ctx.status.map(String::from),
        detail: detail.into(),
    });
}
