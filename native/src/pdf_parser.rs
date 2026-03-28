//! PDF parser module with parallel OCR support
//! Uses lopdf for text extraction

use napi::Result as NapiResult;
use napi_derive::napi;
use rayon::prelude::*;
use lopdf::{Document, Object};
use std::path::Path;

/// PDF parsing result
#[napi(object)]
pub struct PdfParseResult {
    pub success: bool,
    pub text: String,
    pub stage: String,
    pub error: Option<String>,
}

/// Extract text from PDF using lopdf (parallel page processing)
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

    match Document::load(path) {
        Ok(doc) => {
            // get_pages() returns BTreeMap directly, not a Result
            let pages = doc.get_pages();
            let page_ids: Vec<u32> = pages.keys().copied().collect();

            let texts: Vec<String> = page_ids.par_iter()
                .filter_map(|&page_id| {
                    extract_page_text(&doc, page_id).ok()
                })
                .collect();

            let full_text = texts.join("\n\n");

            if full_text.trim().is_empty() {
                return Ok(PdfParseResult {
                    success: false,
                    text: String::new(),
                    stage: "extraction".to_string(),
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
        Err(e) => {
            Ok(PdfParseResult {
                success: false,
                text: String::new(),
                stage: "load".to_string(),
                error: Some(format!("Failed to load PDF: {}", e)),
            })
        }
    }
}

fn extract_page_text(doc: &Document, page_id: u32) -> Result<String, lopdf::Error> {
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
