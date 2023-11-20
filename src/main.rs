use prism::parse as parse_ruby;
use std::collections::HashMap;
use std::path::Path;
use std::{env, fs, io};
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
            .log_message(MessageType::INFO, "fast_ruby_lsp server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
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
    let dir = env::var("project_dir").unwrap_or_else(|_| {
        panic!("missing project_dir env variable!");
    });
    let project_path = Path::new(&dir);
    let mut files = HashMap::new();
    let mut asts = HashMap::new();

    let _ = visit_project_files(project_path, |path| {
        files.insert(path.to_owned(), fs::read(path).unwrap());
    });

    println!("finish loading project files: {:?}", files.len());

    for k in files.keys() {
        asts.insert(k.clone(), parse_ruby(files.get(k).unwrap().as_slice()));
    }
    println!("parse ruby files: {:?}", asts.len());

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
