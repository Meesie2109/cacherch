use cacherch::errors::CacherchError;
use cacherch::{helpers::extract_pdf_text, log::LogStyle};
use clap::{Parser, Subcommand};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io::ErrorKind, path::Path, time::Instant};
use tantivy::Document;
use tantivy::{Index, IndexWriter, ReloadPolicy, doc, schema::*};

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
async fn main() -> Result<(), CacherchError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { path } => index_dir(&path)?,
        Commands::Search { query } => search_query(&query).await?,
    }

    Ok(())
}

fn index_dir(path: &str) -> Result<(), CacherchError> {
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
        Index::open_in_dir(INDEX_DIR)?
    } else {
        Index::create_in_dir(INDEX_DIR, schema)?
    };

    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    fn visit_dir(
        path: &Path,
        title: Field,
        body: Field,
        path_field: Field,
        index_writer: &mut IndexWriter,
    ) -> Result<(), CacherchError> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                visit_dir(&entry_path, title, body, path_field, index_writer)?;
            } else if entry_path.is_file() {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                let content = match ext.as_str() {
                    "txt" => fs::read_to_string(&entry_path)?,
                    "pdf" => entry_path
                        .to_str()
                        .ok_or_else(|| {
                            CacherchError::PdfExtraction(entry_path.display().to_string())
                        })
                        .and_then(|s| {
                            extract_pdf_text(s)
                                .map_err(|_| CacherchError::PdfExtraction(s.to_string()))
                        })?,
                    _ => return Err(CacherchError::UnsupportedExtension(ext)),
                };

                let title_str = entry_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_default()
                    .to_string();
                let path_str = entry_path.display().to_string();

                let doc = doc!(title => title_str, body => content, path_field => path_str);
                index_writer.add_document(doc)?;
            }
        }
        Ok(())
    }

    visit_dir(Path::new(path), title, body, path_field, &mut index_writer)?;

    index_writer.commit()?;
    println!("{}", LogStyle::success("Indexing complete."));

    Ok(())
}

async fn search_query(query_str: &str) -> Result<(), CacherchError> {
    let client = redis::Client::open(REDIS_URL)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    let cache_key = format!("query:{}", query_str);
    if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
        println!("{}", LogStyle::info("[Cache Hit]"));
        let results: Vec<SearchResult> = serde_json::from_str(&cached)?;
        for (i, res) in results.iter().enumerate() {
            println!("{}. {} ({:.2}) - {}", i + 1, res.title, res.score, res.path);
        }
        return Ok(());
    }

    println!(
        "{}",
        LogStyle::info("[Cache Miss]: Running Tantivy search...")
    );
    let start = Instant::now();

    let index = Index::open_in_dir(INDEX_DIR)?;
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;
    let searcher = reader.searcher();

    let schema = index.schema();
    let title = schema.get_field("title").or_else(|_| {
        Err(CacherchError::Tantivy(tantivy::TantivyError::SchemaError(
            "Field 'title' not found".into(),
        )))
    })?;
    let body = schema.get_field("body").or_else(|_| {
        Err(CacherchError::Tantivy(tantivy::TantivyError::SchemaError(
            "Field 'body' not found".into(),
        )))
    })?;

    let query_parser = tantivy::query::QueryParser::for_index(&index, vec![title, body]);
    let query = query_parser.parse_query(query_str).map_err(|e| {
        CacherchError::Io(std::io::Error::new(
            ErrorKind::Other,
            format!("Query parsing error: {}", e),
        ))
    })?;
    let top_docs = searcher.search(&query, &tantivy::collector::TopDocs::with_limit(5))?;

    println!("Top {} results for query '{}':", top_docs.len(), query_str);

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let doc: TantivyDocument = searcher.doc(doc_address)?;

        let json_value: Value = serde_json::from_str(&doc.to_json(&schema))?;

        let title_val = json_value
            .get("title")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let path_val = json_value
            .get("path")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        results.push(SearchResult {
            title: title_val,
            path: path_val,
            score,
        });
    }

    let duration = start.elapsed();

    println!(
        "{}",
        LogStyle::info(&format!("Result: {}", duration.as_millis()))
    );

    for (i, res) in results.iter().enumerate() {
        println!("{}. {} ({:.2}) - {}", i + 1, res.title, res.score, res.path);
    }

    let json = serde_json::to_string(&results)?;
    let _: () = conn.set_ex(&cache_key, json, 30).await?;

    println!(
        "{}",
        LogStyle::success("Cached results for the coming 30 seconds")
    );

    Ok(())
}
