//! PDF parser module with three-tier extraction strategy
//! 
//! Tier 1: lopdf (fastest, ~20ms) - Simple text PDFs
//! Tier 2: pdf-extract (fast, ~50-100ms) - Encoded fonts, CMaps
//! Tier 3: LM Studio (fallback via TypeScript) - Complex/scanned PDFs

use napi::Result as NapiResult;
use napi_derive::napi;
use std::path::Path;

/// PDF parsing result
#[napi(object)]
pub struct PdfParseResult {
    pub success: bool,
    pub text: String,
    pub stage: String,
    pub error: Option<String>,
}

/// Extract text from PDF using three-tier approach
#[napi]
pub fn extract_pdf_text(path: String) -> NapiResult<PdfParseResult> {
    let path = Path::new(&path);

    if !path.exists() {
        return Ok(PdfParseResult {
            success: false,
            text: String::new(),
            stage: "load".to_string(),
            error: Some("File does not exist".to_string()),
        });
    }

    // ========== TIER 1: lopdf (fastest, ~20ms) ==========
    // Try basic extraction first - works for simple text PDFs
    if let Ok(result) = extract_with_lopdf(path) {
        if result.success && result.text.len() > 50 {
            return Ok(result);
        }
    }

    // ========== TIER 2: pdf-extract (fast, ~50-100ms) ==========
    // Better extraction with encoding/CMap support
    match extract_with_pdf_extract(path) {
        Ok(text) if text.len() > 50 => {
            return Ok(PdfParseResult {
                success: true,
                text,
                stage: "pdf-extract".to_string(),
                error: None,
            });
        }
        _ => {}
    }

    // ========== TIER 3: LM Studio (TypeScript fallback) ==========
    // Let TypeScript handle LM Studio parser for complex PDFs
    Ok(PdfParseResult {
        success: false,
        text: String::new(),
        stage: "rust-fallback".to_string(),
        error: Some("No text extracted - use LM Studio fallback".to_string()),
    })
}

/// Tier 1: Basic PDF extraction using lopdf
fn extract_with_lopdf(path: &Path) -> Result<PdfParseResult, String> {
    use lopdf::{Document, Object};

    let doc = Document::load(path)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    let pages = doc.get_pages();
    let page_ids: Vec<u32> = pages.keys().copied().collect();

    let texts: Vec<String> = page_ids.iter()
        .filter_map(|&page_id| {
            extract_page_text_lopdf(&doc, page_id).ok()
        })
        .collect();

    let full_text = texts.join("\n\n");

    if full_text.trim().is_empty() {
        return Ok(PdfParseResult {
            success: false,
            text: String::new(),
            stage: "lopdf".to_string(),
            error: Some("No text extracted".to_string()),
        });
    }

    Ok(PdfParseResult {
        success: true,
        text: full_text,
        stage: "lopdf".to_string(),
        error: None,
    })
}

/// Extract text from a single page using lopdf
fn extract_page_text_lopdf(doc: &lopdf::Document, page_id: u32) -> Result<String, lopdf::Error> {
    use lopdf::Object;

    let page = doc.get_object((page_id, 0))?;
    let page_dict = page.as_dict()?;
    let contents = page_dict.get(b"Contents")?;

    let mut text = String::new();

    match contents {
        Object::Stream(stream) => {
            let content_data = stream.decompressed_content()?;
            text = extract_text_from_content(&content_data);
        }
        Object::Array(arr) => {
            for obj_ref in arr {
                if let Object::Reference(id) = obj_ref {
                    if let Ok(obj) = doc.get_object(*id) {
                        if let Object::Stream(stream) = obj {
                            if let Ok(data) = stream.decompressed_content() {
                                text.push_str(&extract_text_from_content(&data));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(text)
}

/// Extract text from PDF content stream (lopdf)
fn extract_text_from_content(content: &[u8]) -> String {
    let mut text = String::new();
    let mut in_text = false;
    let mut current_text = Vec::new();

    for &byte in content {
        if byte == b'(' && !in_text {
            in_text = true;
            current_text.clear();
        } else if byte == b')' && in_text {
            in_text = false;
            let decoded = String::from_utf8(current_text.clone())
                .unwrap_or_else(|_| {
                    current_text.iter()
                        .filter(|&&b| b >= 32 && b < 127)
                        .map(|&b| b as char)
                        .collect()
                });
            if !decoded.trim().is_empty() {
                text.push_str(&decoded);
                text.push(' ');
            }
            current_text.clear();
        } else if in_text {
            current_text.push(byte);
        }
    }

    text.trim().to_string()
}

/// Tier 2: Better PDF extraction using pdf-extract crate
fn extract_with_pdf_extract(path: &Path) -> Result<String, String> {
    use pdf_extract::extract_text;

    let text = extract_text(path)
        .map_err(|e| format!("pdf-extract error: {}", e))?;

    // Clean up excessive whitespace
    let cleaned = text
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(cleaned)
}

/// PDF OCR result
#[napi(object)]
pub struct PdfOcrResult {
    pub success: bool,
    pub text: String,
    pub pages_processed: u32,
    pub error: Option<String>,
}

/// OCR for PDF pages - delegates to OCR module
#[napi]
pub fn ocr_pdf_pages(_path: String, max_pages: u32, _lang: String) -> NapiResult<PdfOcrResult> {
    Ok(PdfOcrResult {
        success: false,
        text: String::new(),
        pages_processed: max_pages,
        error: Some("PDF OCR requires image extraction - use image OCR directly".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pdf_text_nonexistent() {
        let result = extract_pdf_text("nonexistent.pdf".to_string()).unwrap();
        assert!(!result.success);
        assert_eq!(result.stage, "load");
    }
}
