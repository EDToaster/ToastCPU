//! Language server for ToastLang (`.tl`).
//!
//! Provides live **diagnostics** (include-aware), semantic **highlighting**,
//! **go-to-definition**, and **stack inspection** (hover + `??` typed holes),
//! all driven by the shared compiler frontend (`tlc`). See `README.md` for the
//! roadmap of remaining features.

mod definition;
mod semantic;
mod stack;

use std::path::Path;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    /// Full text of every open document, keyed by URI.
    documents: DashMap<Url, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "language-server-tl".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                // We resend the full document on every change (simple + robust
                // for milestone 1; incremental sync is a later optimization).
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic::legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                        },
                    ),
                ),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "toastlang language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.documents.insert(doc.uri.clone(), doc.text.clone());
        self.publish_diagnostics(doc.uri, &doc.text, Some(doc.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // With FULL sync the last content change carries the whole document.
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.insert(uri.clone(), change.text.clone());
            self.publish_diagnostics(uri, &change.text, Some(params.text_document.version))
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        // Clear diagnostics for the now-closed file.
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        let locations = definition::goto(&uri, &text, position);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        Ok(stack::hover(&uri, &text, position))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };
        let (ast, _diags) = tlc::parse(&text);
        let data = semantic::semantic_tokens(&text, ast.as_ref());
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }
}

impl Backend {
    /// Analyze `text` and publish diagnostics for `uri`.
    ///
    /// Two layers, both with correct positions for the document as typed:
    /// 1. **Lex/parse** diagnostics from the raw document (`#include` lines lex
    ///    as comments, so their offsets are already correct).
    /// 2. **Type/stack-effect** diagnostics from an include-expanded, source-
    ///    mapped analysis ([`tlc::check_mapped`]), keeping only those that map
    ///    back to *this* document. This is what lets type errors surface on
    ///    files that use `#include`.
    async fn publish_diagnostics(&self, uri: Url, text: &str, version: Option<i32>) {
        let (ast, frontend_diags) = tlc::parse(text);
        let mut diags: Vec<Diagnostic> = frontend_diags
            .iter()
            .map(|d| to_lsp_diagnostic(text, d))
            .collect();

        match uri.to_file_path() {
            Ok(path) => {
                let doc_path = path.to_string_lossy().into_owned();
                let include_paths = include_paths_for(&path);
                let analysis = tlc::analyze_mapped(text, &doc_path, &include_paths);
                // Type/stack-effect errors located in this document (mapped
                // offsets are into `text`); errors inside includes are theirs.
                for md in &analysis.diagnostics {
                    if md.file == doc_path {
                        diags.push(mapped_to_lsp_diagnostic(text, md));
                    }
                }
                // `??` holes: show the stack there as an unobtrusive hint.
                for probe in &analysis.probes {
                    if probe.is_hole && probe.file == doc_path {
                        diags.push(hole_diagnostic(text, probe));
                    }
                }
            }
            Err(_) => {
                // Non-file buffer (e.g. unsaved): we can't resolve includes, so
                // fall back to include-free type checking.
                if !text.contains("#include<") {
                    if let Some(module) = &ast {
                        for d in tlc::check(module) {
                            diags.push(to_lsp_diagnostic(text, &d));
                        }
                    }
                }
            }
        }

        self.client.publish_diagnostics(uri, diags, version).await;
    }
}

/// Include search paths for a document, mirroring the project's compiler
/// convention (`-I../../lib/ -I./`): the file's own directory, plus any ancestor
/// directory that contains a `lib/` subdirectory.
fn include_paths_for(doc: &Path) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    if let Some(dir) = doc.parent() {
        paths.push(dir.to_string_lossy().into_owned());
    }
    for ancestor in doc.ancestors() {
        let lib = ancestor.join("lib");
        if lib.is_dir() {
            let s = lib.to_string_lossy().into_owned();
            if !paths.contains(&s) {
                paths.push(s);
            }
        }
    }
    paths
}

/// Convert a frontend [`tlc::Diagnostic`] (byte-offset span) to an LSP one.
fn to_lsp_diagnostic(source: &str, d: &tlc::Diagnostic) -> Diagnostic {
    let severity = match d.severity {
        tlc::Severity::Error => DiagnosticSeverity::ERROR,
        tlc::Severity::Warning => DiagnosticSeverity::WARNING,
    };
    Diagnostic {
        range: Range {
            start: byte_to_position(source, d.start),
            end: byte_to_position(source, d.end),
        },
        severity: Some(severity),
        source: Some("toastlang".to_string()),
        message: d.message.clone(),
        ..Diagnostic::default()
    }
}

/// Convert a mapped frontend diagnostic (byte offsets into the document text)
/// to an LSP diagnostic.
fn mapped_to_lsp_diagnostic(source: &str, d: &tlc::MappedDiagnostic) -> Diagnostic {
    let severity = match d.severity {
        tlc::Severity::Error => DiagnosticSeverity::ERROR,
        tlc::Severity::Warning => DiagnosticSeverity::WARNING,
    };
    Diagnostic {
        range: Range {
            start: byte_to_position(source, d.start),
            end: byte_to_position(source, d.end),
        },
        severity: Some(severity),
        source: Some("toastlang".to_string()),
        message: d.message.clone(),
        ..Diagnostic::default()
    }
}

/// Build an unobtrusive info diagnostic showing the stack at a `??` hole.
fn hole_diagnostic(source: &str, probe: &tlc::StackInfo) -> Diagnostic {
    let stack = if probe.stack.is_empty() {
        "‹empty›".to_string()
    } else {
        probe.stack.clone()
    };
    Diagnostic {
        range: Range {
            start: byte_to_position(source, probe.start),
            end: byte_to_position(source, probe.end),
        },
        severity: Some(DiagnosticSeverity::INFORMATION),
        source: Some("toastlang".to_string()),
        message: format!("?? — stack: {stack}"),
        ..Diagnostic::default()
    }
}

/// Convert a byte offset into `source` to an LSP [`Position`] (0-based line,
/// 0-based UTF-16 character offset, per the LSP spec's default encoding).
fn byte_to_position(source: &str, byte: usize) -> Position {
    let byte = byte.min(source.len());
    let mut line = 0u32;
    let mut character = 0u32;
    let mut idx = 0usize;
    for ch in source.chars() {
        if idx >= byte {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
        idx += ch.len_utf8();
    }
    Position { line, character }
}

/// Convert an LSP [`Position`] (0-based line, UTF-16 column) to a byte offset
/// into `source`. Inverse of [`byte_to_position`].
fn position_to_byte(source: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in source.char_indices() {
        if line == pos.line && col >= pos.character {
            return i;
        }
        if line > pos.line {
            return i;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    source.len()
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
