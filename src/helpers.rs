use std::io::ErrorKind;

use pdf_extract::extract_text;

use crate::log::LogStyle;

pub fn extract_pdf_text(path: &str) -> std::io::Result<String> {
    match extract_text(path) {
        Ok(text) => Ok(text),
        Err(e) => {
            println!(
                "{}",
                LogStyle::error(&format!("Could not extract PDF text {}: {}", path, e))
            );
            Err(std::io::Error::new(ErrorKind::Other, e.to_string()))
        }
    }
}
