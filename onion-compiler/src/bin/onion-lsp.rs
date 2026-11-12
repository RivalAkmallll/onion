use onion_compiler::analyze;
use onion_compiler::error::OnionError;
use onion_compiler::lexer::{lex, Token};
use std::collections::HashMap;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    /// Latest full text of every open file.
    docs: Mutex<HashMap<Url, String>>,
}

impl Backend {
    /// Store the new text, re-check it, send squiggles to the editor.
    async fn refresh(&self, uri: Url, text: String, version: Option<i32>) {
        let diagnostics: Vec<Diagnostic> = analyze(&text)
            .iter()
            .map(|error| to_diagnostic(&text, error))
            .collect();

        self.docs.lock().unwrap().insert(uri.clone(), text);
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "onion-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "onion-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.refresh(doc.uri, doc.text, Some(doc.version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let doc = params.text_document;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.refresh(doc.uri, change.text, Some(doc.version)).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.docs.lock().unwrap().remove(&uri);
        // Clear squiggles for the closed file.
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let text = self
            .docs
            .lock()
            .unwrap()
            .get(&uri)
            .cloned()
            .unwrap_or_default();
        Ok(Some(CompletionResponse::Array(completions(&text))))
    }
}

// ------------------------------------------------------------ helpers

/// Byte offset -> LSP position (line, UTF-16 column).
fn position(source: &str, offset: usize) -> Position {
    let mut offset = offset.min(source.len());
    while !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &source[..offset];
    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let character = source[line_start..offset].encode_utf16().count() as u32;
    Position { line, character }
}

fn to_diagnostic(source: &str, error: &OnionError) -> Diagnostic {
    let range = match &error.span {
        Some(span) => Range {
            start: position(source, span.start),
            end: position(source, span.end.max(span.start)),
        },
        None => Range::default(),
    };

    let mut message = error.message.clone();
    if let Some(help) = &error.help {
        message.push_str(&format!("\nhelp: {help}"));
    }

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("onion".to_string()),
        message,
        ..Default::default()
    }
}

/// Names declared with `let` / `let mut` in the file.
fn variables_in(source: &str) -> Vec<String> {
    let Ok(tokens) = lex(source) else {
        return Vec::new();
    };

    let mut names: Vec<String> = Vec::new();
    for i in 0..tokens.len() {
        if tokens[i].0 != Token::Let {
            continue;
        }
        let mut j = i + 1;
        if matches!(tokens.get(j), Some((Token::Mut, _))) {
            j += 1;
        }
        if let Some((Token::Ident(name), _)) = tokens.get(j) {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    names
}

fn completions(source: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    for keyword in [
        "fn", "let", "mut", "if", "else", "loop", "break", "continue", "true", "false",
    ] {
        items.push(CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for ty in ["i32", "f64", "bool", "String"] {
        items.push(CompletionItem {
            label: ty.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("built-in type".to_string()),
            ..Default::default()
        });
    }

    let snippets: [(&str, &str, &str); 4] = [
        ("println!", r#"println!("${1}")"#, "print a line"),
        ("fn main", "fn main() {\n\t$0\n}", "program entry point"),
        ("if block", "if ${1:condition} {\n\t$0\n}", "if statement"),
        ("loop block", "loop {\n\t$0\n}", "endless loop"),
    ];
    for (label, body, detail) in snippets {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some(detail.to_string()),
            insert_text: Some(body.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    for name in variables_in(source) {
        items.push(CompletionItem {
            label: name,
            kind: Some(CompletionItemKind::VARIABLE),
            ..Default::default()
        });
    }

    items
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: Mutex::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
