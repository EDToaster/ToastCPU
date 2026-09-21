use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        // Smoke test: prove the shared `tlc` frontend is reachable as a library.
        let diagnostic_count = frontend_smoke_test();
        self.client
            .log_message(
                MessageType::INFO,
                format!("server initialized! frontend produced {diagnostic_count} diagnostic(s) on the smoke-test snippet"),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// Minimal smoke test proving the LSP crate can call into the shared `tlc`
/// frontend library: parse a snippet and return the diagnostic count.
fn frontend_smoke_test() -> usize {
    let source = "fn main -> { 1 2 + }";
    let analysis = tlc::analyze(source, &[]);
    analysis.diagnostics.len()
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}