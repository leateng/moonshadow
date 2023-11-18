use prism::{parse, ParseResult};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use walkdir::WalkDir;

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
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

fn visit_ruby_files<F>(dir: &Path, mut callback: F) -> io::Result<()>
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

// fn read_file(path: &Path) -> Result<&[u8], std::io::Error> {
//     let file_content = fs::read(path)?;
//
//     Ok(&file_content)
// }

struct SourceCode<'a> {
    source: Vec<u8>,
    parse: ParseResult<'a>,
}

#[tokio::main]
async fn main() {
    let mut asts = HashMap::new();
    let project_path = Path::new("D:\\work\\CRM_NEW");
    visit_ruby_files(project_path, |path| {
        println!("Found ruby file: {:?}", path);
        let content = Box::new(fs::read(path).unwrap());
        asts.insert(
            path.to_owned(),
            SourceCode {
                source: *content,
                parse: parse(&(*content.as_slice())),
            },
        );
    });

    // println!("{:?}", asts.len());
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
