use std::{fs, io::ErrorKind, path::Path};

use cacherch::{helpers::extract_pdf_text, log::LogStyle};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tantivy::{Index, IndexWriter, doc, schema::*};

#[derive(Parser)]
#[command(
    name = "cacherch",
    version,
    about = "Mini search engine with Redis caching"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index { path: String },
    Search { query: String },
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    path: String,
    score: f32,
}

const INDEX_DIR: &str = "./index";
const REDIS_URL: &str = "redis://127.0.0.1/";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { path } => index_dir(&path)?,
        Commands::Search { query } => search_query(&query).await?,
    }

    Ok(())
}

fn index_dir(path: &str) -> std::io::Result<()> {
    println!(
        "{}",
        LogStyle::info(&format!("Indexing directory: {}", path))
    );

    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let body = schema_builder.add_text_field("body", TEXT);
    let path_field = schema_builder.add_text_field("path", STORED);
    let schema = schema_builder.build();

    std::fs::create_dir_all(INDEX_DIR)?;
    let index = if Path::new(INDEX_DIR).exists() {
        Index::open_in_dir(INDEX_DIR)
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?
    } else {
        Index::create_in_dir(INDEX_DIR, schema)
            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?
    };

    let mut index_writer: IndexWriter = index
        .writer(50_000_000)
        .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;

    fn visit_dir(
        path: &Path,
        title: Field,
        body: Field,
        path_field: Field,
        index_writer: &mut IndexWriter,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                visit_dir(&entry_path, title, body, path_field, index_writer)?;
            } else if entry_path.is_file()
                && entry_path.extension().and_then(|e| e.to_str()) == Some("txt")
            {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                let content_res: Result<String, std::io::Error> = match ext.as_str() {
                    "txt" => fs::read_to_string(&entry_path),
                    "pdf" => entry_path
                        .to_str()
                        .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "Invalid PDF path"))
                        .and_then(|s| {
                            extract_pdf_text(s).or_else(|_| {
                                Err(std::io::Error::new(
                                    ErrorKind::Other,
                                    format!("Failed to extract PDF: {}", s),
                                ))
                            })
                        }),
                    _ => Err(std::io::Error::new(
                        ErrorKind::Other,
                        format!("File extension not supported: {}", ext),
                    )),
                };

                match content_res {
                    Ok(content) => {
                        let title_str = entry_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| {
                                println!(
                                    "{}",
                                    LogStyle::warning(&format!(
                                        "Skipping file with invalid name: {}",
                                        entry_path.display()
                                    ))
                                );
                                "".to_string()
                            });

                        let path_str = entry_path.display().to_string();

                        let doc = doc!(
                            title => title_str,
                            body => content,
                            path_field => path_str
                        );

                        index_writer
                            .add_document(doc)
                            .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;
                    }
                    Err(e) => println!("{}", LogStyle::error(&format!("{}", e))),
                }
            }
        }
        Ok(())
    }

    visit_dir(Path::new(path), title, body, path_field, &mut index_writer)?;

    index_writer
        .commit()
        .map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;
    println!("{}", LogStyle::success("Indexing complete."));

    Ok(())
}

async fn search_query(query_str: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(
        ErrorKind::Unsupported,
        "Function not implemented".to_string(),
    ))
}
