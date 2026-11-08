use std::io::Write;
use std::sync::LazyLock;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

use crate::update_check::{UpdateCheckError, UpdateChecker};
use crate::xbps_template::XBPSTemplate;
use crate::xlint::XlintDiagnosticsEngine;

mod bash_executor;
mod manual;
mod markdown;
mod update_check;
mod xbps_template;
mod xlint;

static MANUAL: LazyLock<manual::Manual> = LazyLock::new(manual::Manual::default);

#[derive(Debug)]
struct Backend {
    client: Client,
}

fn range_from_lineno(lineno: u32) -> Range {
    Range {
        start: Position {
            line: lineno,
            character: 0,
        },
        end: Position {
            line: lineno,
            character: 0,
        },
    }
}

impl Backend {
    fn completions(&self) -> Vec<CompletionItem> {
        let template_variables = &MANUAL.template_variables;
        let template_variables = template_variables.iter().map(|v| CompletionItem {
            label: v.name.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: v.description.clone(),
            })),
            insert_text: Some(format!("{}=", v.name.clone())),
            ..Default::default()
        });

        let template_functions = &MANUAL.template_functions;
        let template_functions = template_functions.iter().map(|f| CompletionItem {
            label: f.name.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: f.description.clone(),
            })),
            insert_text: Some(format!("{} {{\n}}", f.name.clone())),
            ..Default::default()
        });

        let global_variables = &MANUAL.global_variables;
        let global_variables = global_variables.iter().map(|v| CompletionItem {
            label: v.name.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: v.description.clone(),
            })),
            ..Default::default()
        });

        let global_functions = &MANUAL.global_functions;
        let global_functions = global_functions.iter().map(|f| CompletionItem {
            label: f.name.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: f.description.clone(),
            })),
            insert_text: Some(f.template.clone()),
            ..Default::default()
        });

        let build_styles = &MANUAL.build_styles;
        let build_styles = build_styles.iter().map(|b| CompletionItem {
            label: b.name.clone(),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: b.description.clone(),
            })),
            ..Default::default()
        });

        template_variables
            .chain(template_functions)
            .chain(global_variables)
            .chain(global_functions)
            .chain(build_styles)
            .collect::<Vec<_>>()
    }

    fn xlint_diagnostics(&self, path: std::path::PathBuf) -> Vec<Diagnostic> {
        let xlint_engine = XlintDiagnosticsEngine::new(path).unwrap();
        xlint_engine
            .diagnostics()
            .unwrap()
            .iter()
            .map(|(lineno, desc)| Diagnostic {
                range: range_from_lineno(*lineno),
                message: desc.to_string(),
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            })
            .collect()
    }

    async fn update_diagnostics(&self, uri: Uri, text: String) {
        let mut tempfile = tempfile::NamedTempFile::new().unwrap();
        tempfile.write_all(&text.clone().into_bytes()).unwrap();

        let mut diagnostics = self.xlint_diagnostics(tempfile.path().to_path_buf());

        self.client
            .publish_diagnostics(uri.clone(), diagnostics.clone(), None)
            .await;

        let update_checker = UpdateChecker::new(text, uri.to_file_path().unwrap().to_path_buf());

        let Some((version_lineno, _)) = update_checker.current_version() else {
            return;
        };
        // TODO: cache latest version, so that this doesn't get run each time the file changes
        match update_checker.new_version() {
            Ok(Some(version)) => {
                diagnostics.push(Diagnostic {
                    range: range_from_lineno(version_lineno as u32),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: format!("new version available: {version}"),
                    ..Default::default()
                });

                self.client
                    .publish_diagnostics(uri, diagnostics, None)
                    .await;
            }
            Err(UpdateCheckError::NoVersionFound) => {
                diagnostics.push(Diagnostic {
                    range: range_from_lineno(version_lineno as u32),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "no version found, please override the `update` file".to_string(),
                    ..Default::default()
                });

                self.client
                    .publish_diagnostics(uri, diagnostics, None)
                    .await;
            }
            _ => {}
        }
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions::default(),
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_diagnostics(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let text = params.content_changes[0].text.clone();
        self.update_diagnostics(params.text_document.uri, text)
            .await;
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let path = params.text_document.uri.to_file_path().unwrap();
        let diagnostics = self.xlint_diagnostics(path.to_path_buf());

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    items: diagnostics,
                    ..Default::default()
                },
                ..Default::default()
            }),
        ))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let doc = XBPSTemplate::from_path(
            &params
                .text_document_position_params
                .text_document
                .uri
                .to_file_path()
                .unwrap(),
        )
        .unwrap();

        let Position { line, character } = params.text_document_position_params.position;
        let Some((identifier, start, end)) = doc.identifier_at_pos(line, character) else {
            return Ok(None);
        };

        for completion in self.completions() {
            if completion.label == identifier {
                let Some(Documentation::MarkupContent(documentation)) = completion.documentation
                else {
                    return Ok(None);
                };
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(documentation),
                    range: Some(tower_lsp_server::ls_types::Range {
                        start: Position {
                            line,
                            character: start,
                        },
                        end: Position {
                            line,
                            character: end,
                        },
                    }),
                }));
            }
        }

        Ok(None)
    }

    async fn completion(
        &self,
        _: tower_lsp_server::ls_types::CompletionParams,
    ) -> Result<Option<tower_lsp_server::ls_types::CompletionResponse>> {
        Ok(Some(self.completions().into()))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
