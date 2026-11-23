//! UEAS Language Server Protocol (LSP) server powered by `ueas-syntax`, `ueas-hir`, and `tower-lsp`.
#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use ueas_hir::{compile_to_hir_detailed, EmbeddedStdlibResolver};
use ueas_syntax::{format_source, parse};

struct Backend {
    client: Client,
    docs: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: String) {
        let mut diags = Vec::new();
        if let Err(items) = compile_to_hir_detailed(&text, &EmbeddedStdlibResolver) {
            for item in items {
                let range = match item.span {
                    Some(sp) => Range::new(
                        Position::new(
                            sp.start_line.saturating_sub(1) as u32,
                            sp.start_col.saturating_sub(1) as u32,
                        ),
                        Position::new(
                            sp.end_line.saturating_sub(1) as u32,
                            sp.end_col.saturating_sub(1) as u32,
                        ),
                    ),
                    None => Range::new(Position::new(0, 0), Position::new(0, 1)),
                };
                diags.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(item.code)),
                    source: Some("ueas-lsp".to_string()),
                    message: item.message,
                    ..Diagnostic::default()
                });
            }
        }
        let _ = parse(&text);
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..CompletionOptions::default()
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "ueas-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ueas-lsp v2.1 initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.docs.write().await.insert(uri.clone(), text.clone());
        self.validate_document(uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.docs.write().await.insert(uri.clone(), text.clone());
            self.validate_document(uri, text).await;
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let guard = self.docs.read().await;
        let Some(text) = guard.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let Ok(formatted) = format_source(text) else {
            return Ok(None);
        };
        let line_count = text.lines().count() as u32 + 1;
        Ok(Some(vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(line_count, 0)),
            new_text: formatted,
        }]))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let guard = self.docs.read().await;
        let Some(text) = guard.get(&uri) else {
            return Ok(None);
        };
        let Ok(prog) = compile_to_hir_detailed(text, &EmbeddedStdlibResolver) else {
            return Ok(None);
        };
        let line = (pos.line + 1) as usize;
        for algo in &prog.algorithms {
            if line >= algo.span.start_line && line <= algo.span.end_line {
                let time = algo
                    .complexity
                    .time
                    .as_ref()
                    .map_or("unspecified", |b| b.raw.as_str());
                let md = format!(
                    "**Algorithm `{}`**\n- Time: `{}`\n- ConstantTime: `{}`",
                    algo.name, time, algo.constant_time
                );
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: md,
                    }),
                    range: None,
                }));
            }
        }
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let guard = self.docs.read().await;
        let Some(text) = guard.get(&uri) else {
            return Ok(None);
        };
        let word = word_at_position(text, pos.line as usize, pos.character as usize);
        if word.is_empty() {
            return Ok(None);
        }
        let Ok(prog) = compile_to_hir_detailed(text, &EmbeddedStdlibResolver) else {
            return Ok(None);
        };
        for algo in &prog.algorithms {
            if algo.name == word {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    uri.clone(),
                    Range::new(
                        Position::new(algo.span.start_line.saturating_sub(1) as u32, 0),
                        Position::new(algo.span.start_line.saturating_sub(1) as u32, 1),
                    ),
                ))));
            }
        }
        for rec in &prog.records {
            if rec.name == word {
                return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                    uri.clone(),
                    Range::new(
                        Position::new(rec.span.start_line.saturating_sub(1) as u32, 0),
                        Position::new(rec.span.start_line.saturating_sub(1) as u32, 1),
                    ),
                ))));
            }
        }
        Ok(None)
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let keywords = [
            "Algorithm",
            "Record",
            "Require:",
            "Ensure(res):",
            "Complexity:",
            "let",
            "let mut",
            "const",
            "if",
            "else if",
            "while",
            "for",
            "parallel for",
            "return",
            "assert",
            "invariant",
            "decreases",
            "select",
            "forall",
            "exists",
            "import",
            "module",
        ];
        let types = [
            "Int",
            "Int64",
            "UInt64",
            "Real",
            "Bool",
            "String",
            "Void",
            "List<Int>",
            "Option<Int>",
            "Result<Int, String>",
            "Map<Int, Int>",
            "Set<Int>",
            "Secret<Int>",
        ];
        let mut items: Vec<CompletionItem> = keywords
            .iter()
            .map(|k| CompletionItem {
                label: (*k).to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..CompletionItem::default()
            })
            .collect();
        items.extend(types.iter().map(|t| CompletionItem {
            label: (*t).to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            ..CompletionItem::default()
        }));
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let guard = self.docs.read().await;
        let Some(text) = guard.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let Ok(prog) = compile_to_hir_detailed(text, &EmbeddedStdlibResolver) else {
            return Ok(None);
        };
        #[allow(deprecated)]
        let syms: Vec<SymbolInformation> = prog
            .algorithms
            .iter()
            .map(|a| SymbolInformation {
                name: a.name.clone(),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location::new(
                    params.text_document.uri.clone(),
                    Range::new(
                        Position::new(a.span.start_line.saturating_sub(1) as u32, 0),
                        Position::new(a.span.end_line.saturating_sub(1) as u32, 0),
                    ),
                ),
                container_name: prog.module.clone(),
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Flat(syms)))
    }
}

fn word_at_position(text: &str, line_idx: usize, col_idx: usize) -> String {
    let Some(line) = text.lines().nth(line_idx) else {
        return String::new();
    };
    let bytes = line.as_bytes();
    let mut start = col_idx.min(bytes.len());
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    let mut end = col_idx.min(bytes.len());
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    String::from_utf8_lossy(&bytes[start..end]).to_string()
}

/// Run the UEAS LSP server on stdin/stdout.
pub async fn run_lsp_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: Arc::new(RwLock::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
