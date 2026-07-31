//! LSP stdio server wiring analysis results to the protocol.
//!
//! Uses `lsp_types::Uri` as a `HashMap` key. `Uri` contains interior
//! mutability for thread-safe cheap cloning, but we never mutate it
//! after inserting it into the map, so the standard clippy warning
//! is suppressed at module level.

use std::collections::HashMap;
use std::error::Error;

use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverContents, InitializeParams,
    MarkupContent, MarkupKind, NumberOrString, OneOf, Position, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
};

use super::{
    CompletionKind, TexCompletion, TexDiagnostic, TexRange, TexSymbol, analyze_diagnostics,
    command_completions, completions_at, document_symbols, hover_at, range_to_positions,
};

#[allow(clippy::mutable_key_type)]
pub fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();

    let (id, params) = connection.initialize_start()?;
    let _init_params: InitializeParams = serde_json::from_value(params)?;

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                ..Default::default()
            },
        )),
        completion_provider: Some(lsp_types::CompletionOptions {
            trigger_characters: Some(vec!["\\".into(), "{".into()]),
            ..Default::default()
        }),
        document_symbol_provider: Some(OneOf::Left(true)),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        ..Default::default()
    };

    let init = serde_json::json!({
        "capabilities": capabilities,
        "serverInfo": {
            "name": "rtex-lsp",
            "version": env!("CARGO_PKG_VERSION"),
        }
    });
    connection.initialize_finish(id, init)?;

    let mut documents: HashMap<Uri, String> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Notification(Notification { method, params, .. }) => {
                if method == "textDocument/didOpen" {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(params.clone())?;
                    let uri = params.text_document.uri;
                    let text = params.text_document.text;
                    documents.insert(uri.clone(), text.clone());
                    publish_diagnostics(&connection, &uri, &text)?;
                } else if method == "textDocument/didChange" {
                    let params: DidChangeTextDocumentParams =
                        serde_json::from_value(params.clone())?;
                    let uri = params.text_document.uri;
                    if let Some(change) = params.content_changes.into_iter().next() {
                        let text = change.text;
                        documents.insert(uri.clone(), text.clone());
                        publish_diagnostics(&connection, &uri, &text)?;
                    }
                } else if method == "exit" {
                    break;
                }
            }
            Message::Request(Request {
                id, method, params, ..
            }) => {
                let response = match method.as_str() {
                    "shutdown" => Response::new_ok(id.clone(), serde_json::Value::Null),
                    "textDocument/completion" => {
                        let result = handle_completion(&documents, &params)?;
                        Response::new_ok(id.clone(), serde_json::to_value(result)?)
                    }
                    "textDocument/documentSymbol" => {
                        let result = handle_document_symbol(&documents, &params)?;
                        Response::new_ok(id.clone(), serde_json::to_value(result)?)
                    }
                    "textDocument/hover" => {
                        let result = handle_hover(&documents, &params)?;
                        Response::new_ok(id.clone(), serde_json::to_value(result)?)
                    }
                    _ => Response::new_ok(id.clone(), serde_json::Value::Null),
                };
                connection.sender.send(Message::Response(response))?;
            }
            _ => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: &Uri,
    text: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let diagnostics: Vec<Diagnostic> = analyze_diagnostics(text)
        .iter()
        .map(|d| to_lsp_diagnostic(text, d))
        .collect();

    let params = serde_json::json!({
        "uri": uri,
        "diagnostics": diagnostics,
    });

    let notif = Notification {
        method: "textDocument/publishDiagnostics".to_string(),
        params,
    };
    connection.sender.send(Message::Notification(notif))?;
    Ok(())
}

#[allow(clippy::mutable_key_type)]
fn handle_completion(
    documents: &HashMap<Uri, String>,
    params: &serde_json::Value,
) -> Result<Option<CompletionResponse>, Box<dyn Error + Send + Sync>> {
    let uri: Uri = serde_json::from_value(params["textDocument"]["uri"].clone())?;
    let position: Position = serde_json::from_value(params["position"].clone())?;
    let text = documents.get(&uri).ok_or("document not open")?;
    let offset = position_to_offset(text, position.line, position.character);

    let items: Vec<TexCompletion> = completions_at(text, offset);
    let completions: Vec<CompletionItem> = items
        .into_iter()
        .map(|item| CompletionItem {
            label: item.label,
            kind: Some(match item.kind {
                CompletionKind::Command => CompletionItemKind::FUNCTION,
                CompletionKind::Environment => CompletionItemKind::STRUCT,
            }),
            detail: item.detail,
            insert_text: Some(item.insert_text),
            ..Default::default()
        })
        .collect();

    if completions.is_empty() {
        // Fallback: show all commands when triggered with backslash only
        let fallback: Vec<CompletionItem> = command_completions("")
            .into_iter()
            .map(|item| CompletionItem {
                label: item.label,
                kind: Some(CompletionItemKind::FUNCTION),
                detail: item.detail,
                insert_text: Some(item.insert_text),
                ..Default::default()
            })
            .collect();
        return Ok(Some(CompletionResponse::Array(fallback)));
    }

    Ok(Some(CompletionResponse::Array(completions)))
}

#[allow(clippy::mutable_key_type)]
fn handle_document_symbol(
    documents: &HashMap<Uri, String>,
    params: &serde_json::Value,
) -> Result<Vec<lsp_types::DocumentSymbol>, Box<dyn Error + Send + Sync>> {
    let uri: Uri = serde_json::from_value(params["textDocument"]["uri"].clone())?;
    let text = documents.get(&uri).ok_or("document not open")?;
    let symbols = document_symbols(text);
    Ok(symbols.iter().map(|s| to_lsp_symbol(text, s)).collect())
}

#[allow(clippy::mutable_key_type)]
fn handle_hover(
    documents: &HashMap<Uri, String>,
    params: &serde_json::Value,
) -> Result<Option<Hover>, Box<dyn Error + Send + Sync>> {
    let uri: Uri = serde_json::from_value(params["textDocument"]["uri"].clone())?;
    let position: Position = serde_json::from_value(params["position"].clone())?;
    let text = documents.get(&uri).ok_or("document not open")?;
    let offset = position_to_offset(text, position.line, position.character);

    Ok(hover_at(text, offset).map(|content| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: None,
    }))
}

fn to_lsp_diagnostic(text: &str, diag: &TexDiagnostic) -> Diagnostic {
    let (start, end) = range_to_positions(text, diag.range);
    Diagnostic {
        range: Range {
            start: Position {
                line: start.line,
                character: start.character,
            },
            end: Position {
                line: end.line,
                character: end.character,
            },
        },
        severity: Some(match diag.severity {
            super::Severity::Error => DiagnosticSeverity::ERROR,
            super::Severity::Warning => DiagnosticSeverity::WARNING,
            super::Severity::Information => DiagnosticSeverity::INFORMATION,
        }),
        code: diag
            .code
            .as_ref()
            .map(|c| NumberOrString::String(c.clone())),
        message: diag.message.clone(),
        source: Some("rtex".to_string()),
        ..Default::default()
    }
}

fn to_lsp_symbol(text: &str, symbol: &TexSymbol) -> lsp_types::DocumentSymbol {
    let (start, end) = range_to_positions(
        text,
        TexRange::span(
            symbol.range_start,
            symbol.range_end.max(symbol.range_start + 1),
        ),
    );
    #[allow(deprecated)]
    lsp_types::DocumentSymbol {
        name: symbol.name.clone(),
        detail: None,
        kind: match symbol.kind {
            super::SymbolKind::Section => lsp_types::SymbolKind::NAMESPACE,
            super::SymbolKind::Subsection => lsp_types::SymbolKind::STRING,
            super::SymbolKind::Label => lsp_types::SymbolKind::KEY,
        },
        range: Range {
            start: Position {
                line: start.line,
                character: start.character,
            },
            end: Position {
                line: end.line,
                character: end.character,
            },
        },
        selection_range: Range {
            start: Position {
                line: start.line,
                character: start.character,
            },
            end: Position {
                line: end.line,
                character: end.character,
            },
        },
        children: Some(
            symbol
                .children
                .iter()
                .map(|c| to_lsp_symbol(text, c))
                .collect(),
        ),
        tags: None,
        deprecated: None,
    }
}

fn position_to_offset(text: &str, line: u32, character: u32) -> usize {
    let mut current_line = 0u32;
    let mut current_char = 0u32;
    let mut offset = 0usize;

    for ch in text.chars() {
        if current_line == line && current_char == character {
            break;
        }
        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
        offset += ch.len_utf8();
    }

    offset.min(text.len())
}
