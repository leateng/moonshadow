use dashmap::DashMap;
use prism::{parse as parse_ruby, ParseResult};
use std::collections::HashMap;
use std::path::Path;
use std::{default, env, fs, io};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use walkdir::WalkDir;

#[derive(Debug)]
struct Backend {
    client: Client,
    root_path: Option<String>,
    files: Option<DashMap<String, Vec<u8>>>,
    // asts: DashMap<String, ParseResult<'a>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "fast_ruby_lsp server start...")
            .await;
        self.client
            .log_message(MessageType::INFO, format!("InitializeParams={:?}", params))
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF8),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                definition_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::from(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "fast_ruby_lsp".into(),
                version: Some("0.0.1".into()),
            }),
            offset_encoding: Some("utf-8".into()),
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, format!("CompletionParams={:?}", params))
            .await;

        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.client
            .log_message(MessageType::INFO, format!("HoverParams={:?}", params))
            .await;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    /// The [`textDocument/didOpen`] notification is sent from the client to the server to signal
    /// that a new text document has been opened by the client.
    ///
    /// [`textDocument/didOpen`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen
    ///
    /// The document's truth is now managed by the client and the server must not try to read the
    /// document’s truth using the document's URI. "Open" in this sense means it is managed by the
    /// client. It doesn't necessarily mean that its content is presented in an editor.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("DidOpenTextDocumentParams={:?}", params),
            )
            .await;
    }

    /// The [`textDocument/didChange`] notification is sent from the client to the server to signal
    /// changes to a text document.
    ///
    /// [`textDocument/didChange`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_didChange
    ///
    /// This notification will contain a distinct version tag and a list of edits made to the
    /// document for the server to interpret.
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("DidChangeTextDocumentParams={:?}", params),
            )
            .await;
    }

    /// The [`textDocument/willSave`] notification is sent from the client to the server before the
    /// document is actually saved.
    ///
    /// [`textDocument/willSave`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_willSave
    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("WillSaveTextDocumentParams={:?}", params),
            )
            .await;
    }

    /// The [`textDocument/willSaveWaitUntil`] request is sent from the client to the server before
    /// the document is actually saved.
    ///
    /// [`textDocument/willSaveWaitUntil`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_willSaveWaitUntil
    ///
    /// The request can return an array of `TextEdit`s which will be applied to the text document
    /// before it is saved.
    ///
    /// Please note that clients might drop results if computing the text edits took too long or if
    /// a server constantly fails on this request. This is done to keep the save fast and reliable.
    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("WillSaveTextDocumentParams={:?}", params),
            )
            .await;

        Ok(None)
    }

    /// The [`textDocument/didSave`] notification is sent from the client to the server when the
    /// document was saved in the client.
    ///
    /// [`textDocument/didSave`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_didSave
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("DidSaveTextDocumentParams={:?}", params),
            )
            .await;
    }

    /// The [`textDocument/didClose`] notification is sent from the client to the server when the
    /// document got closed in the client.
    ///
    /// [`textDocument/didClose`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_didClose
    ///
    /// The document's truth now exists where the document's URI points to (e.g. if the document's
    /// URI is a file URI, the truth now exists on disk).
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("DidCloseTextDocumentParams={:?}", params),
            )
            .await;
    }

    /// The [`textDocument/definition`] request asks the server for the definition location of a
    /// symbol at a given text document position.
    ///
    /// [`textDocument/definition`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_definition
    ///
    /// # Compatibility
    ///
    /// The [`GotoDefinitionResponse::Link`](lsp_types::GotoDefinitionResponse::Link) return value
    /// was introduced in specification version 3.14.0 and requires client-side support in order to
    /// be used. It can be returned if the client set the following field to `true` in the
    /// [`initialize`](Self::initialize) method:
    ///
    /// ```text
    /// InitializeParams::capabilities::text_document::definition::link_support
    /// ```
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("GotoDefinitionParams={:?}", params),
            )
            .await;

        Ok(None)
    }
}

fn visit_project_files<F>(dir: &Path, mut callback: F) -> io::Result<()>
where
    F: FnMut(&Path),
{
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some("rb".as_ref()) {
            callback(path);
        }
    }
    Ok(())
}

// struct SourceCode<'a> {
//     source: Vec<u8>,
//     parse: ParseResult<'a>,
// }

#[tokio::main]
async fn main() {
    // let dir = env::var("project_dir").unwrap_or_else(|_| {
    //     panic!("missing project_dir env variable!");
    // });
    // let project_path = Path::new(&dir);
    let project_path = Path::new("D:\\work\\CRM_NEW");
    let mut files = HashMap::new();
    let mut asts = HashMap::new();

    let _ = visit_project_files(project_path, |path| {
        files.insert(path.to_owned(), fs::read(path).unwrap());
    });

    // println!("finish loading project files: {:?}", files.len());

    for k in files.keys() {
        asts.insert(k.clone(), parse_ruby(files.get(k).unwrap().as_slice()));
    }
    // println!("parse ruby files: {:?}", asts.len());

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client: client,
        root_path: None::<String>,
        files: None::<DashMap<String, Vec<u8>>>,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
