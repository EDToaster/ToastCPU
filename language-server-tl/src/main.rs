//! Language server for ToastLang (`.tl`).
//!
//! Milestone 1 provides live **diagnostics** and semantic **highlighting**,
//! both driven by the shared compiler frontend (`tlc`). See `README.md` for the
//! roadmap of remaining features.

mod semantic;

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
    /// Lex/parse diagnostics are always published (their spans are correct for
    /// the document as typed). Type/stack-effect diagnostics are only published
    /// when the file has no `#include<...>`: the preprocessor expands includes
    /// in place, which shifts byte offsets so checker spans would land in the
    /// wrong place. Cross-include diagnostics via a source map are a follow-up
    /// (see README).
    async fn publish_diagnostics(&self, uri: Url, text: &str, version: Option<i32>) {
        let (ast, frontend_diags) = tlc::parse(text);
        let mut diags: Vec<Diagnostic> = frontend_diags
            .iter()
            .map(|d| to_lsp_diagnostic(text, d))
            .collect();

        let has_includes = text.contains("#include<");
        if !has_includes {
            if let Some(module) = &ast {
                for d in tlc::check(module) {
                    diags.push(to_lsp_diagnostic(text, &d));
                }
            }
        }

        self.client.publish_diagnostics(uri, diags, version).await;
    }
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
