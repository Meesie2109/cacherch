use serde_json::Value;
use std::{io::ErrorKind, time::Instant};
use tantivy::Document;
use tantivy::{Index, ReloadPolicy, schema::*};

use crate::cache;
use crate::errors::CacherchError;
use crate::log::LogStyle;
use crate::types::SearchResult;

const INDEX_DIR: &str = "./index";

pub async fn search_query(query_str: &str, ttl: &usize) -> Result<(), CacherchError> {
    let mut conn = cache::get_connection().await?;

    let cache_key = format!("query:{}", query_str);
    if let Some(results) = cache::get_cached_results(&mut conn, &cache_key).await? {
        println!("{}", LogStyle::info("[Cache Hit]"));
        for (i, res) in results.iter().enumerate() {
            println!(
                "{}. {} ({:.2}) - {}",
                i + 1,
                res.title(),
                res.score(),
                res.path()
            );
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

        results.push(SearchResult::new(title_val, path_val, score));
    }

    let duration = start.elapsed();

    println!(
        "{}",
        LogStyle::info(&format!("Result: {}", duration.as_millis()))
    );

    for (i, res) in results.iter().enumerate() {
        println!(
            "{}. {} ({:.2}) - {}",
            i + 1,
            res.title(),
            res.score(),
            res.path()
        );
    }

    cache::set_cached_results(&mut conn, &cache_key, &results, *ttl).await?;

    println!(
        "{}",
        LogStyle::success(&format!("Cached results for the coming {} seconds", ttl))
    );

    Ok(())
}
