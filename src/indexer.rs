use std::{fs, path::Path};

use tantivy::{
    Index, IndexWriter, doc,
    schema::{Field, STORED, Schema, TEXT},
};

use crate::{errors::CacherchError, helpers::extract_pdf_text, log::LogStyle};

const INDEX_DIR: &str = "./index";

pub fn index_dir(path: &str) -> Result<(), CacherchError> {
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
