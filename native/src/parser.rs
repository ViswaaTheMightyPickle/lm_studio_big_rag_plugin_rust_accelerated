//! Document parser router
//! Routes document parsing to appropriate parser based on file extension

use napi::Result as NapiResult;
use napi_derive::napi;
use std::path::Path;

/// Document parse result
#[napi(object)]
pub struct DocumentParseResult {
    pub success: bool,
    pub text: String,
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub error: Option<String>,
}

/// Parse a document file based on its extension
/// Routes to appropriate parser: PDF, EPUB, HTML, text, or image
#[napi]
pub fn parse_document(
    path: String,
    enable_ocr: bool,
) -> NapiResult<DocumentParseResult> {
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Ok(DocumentParseResult {
            success: false,
            text: String::new(),
            file_path: path.clone(),
            file_name: String::new(),
            extension: String::new(),
            error: Some("File does not exist".to_string()),
        });
    }
    
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    
    let result = match extension.as_str() {
        "pdf" => {
            // PDF parsing
            match crate::pdf_parser::extract_pdf_text(path.clone()) {
                Ok(pdf_result) => {
                    if pdf_result.success {
                        Ok::<(String, Option<String>), napi::Error>((pdf_result.text, None))
                    } else {
                        Ok((String::new(), pdf_result.error))
                    }
                }
                Err(e) => Ok((String::new(), Some(e.to_string()))),
            }
        }
        
        "epub" => {
            // EPUB parsing
            match crate::epub_parser::extract_epub_text(path.clone()) {
                Ok(epub_result) => {
                    if epub_result.success {
                        Ok((epub_result.text, None))
                    } else {
                        // EPUB Rust parser returns error - use TypeScript fallback message
                        Ok((String::new(), Some("EPUB parsing requires TypeScript fallback".to_string())))
                    }
                }
                Err(e) => Ok((String::new(), Some(e.to_string()))),
            }
        }
        
        "html" | "htm" => {
            // HTML parsing
            match crate::html_parser::parse_html(path.clone()) {
                Ok(html_result) => {
                    if html_result.success {
                        Ok((html_result.text, None))
                    } else {
                        Ok((String::new(), html_result.error))
                    }
                }
                Err(e) => Ok((String::new(), Some(e.to_string()))),
            }
        }
        
        "txt" | "md" | "markdown" | "rtf" => {
            // Text/Markdown parsing
            match crate::text_parser::parse_text_file(path.clone(), None) {
                Ok(text_result) => {
                    if text_result.success {
                        Ok((text_result.text, None))
                    } else {
                        Ok((String::new(), text_result.error))
                    }
                }
                Err(e) => Ok((String::new(), Some(e.to_string()))),
            }
        }
        
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => {
            // Image parsing (OCR)
            if enable_ocr {
                match crate::ocr::ocr_image(path.clone(), None) {
                    Ok(ocr_result) => {
                        if ocr_result.success {
                            Ok((ocr_result.text, None))
                        } else {
                            Ok((String::new(), ocr_result.error))
                        }
                    }
                    Err(e) => Ok((String::new(), Some(e.to_string()))),
                }
            } else {
                Ok((String::new(), Some("OCR is disabled".to_string())))
            }
        }
        
        _ => {
            Ok((String::new(), Some("Unsupported file type".to_string())))
        }
    };
    
    match result {
        Ok((text, error)) => Ok(DocumentParseResult {
            success: !text.is_empty() && error.is_none(),
            text,
            file_path: path,
            file_name,
            extension,
            error,
        }),
        Err(e) => Ok(DocumentParseResult {
            success: false,
            text: String::new(),
            file_path: path,
            file_name,
            extension,
            error: Some(e.to_string()),
        }),
    }
}

/// Parse multiple documents in parallel
#[napi]
pub fn parse_documents_batch(
    paths: Vec<String>,
    enable_ocr: bool,
) -> NapiResult<Vec<DocumentParseResult>> {
    use rayon::prelude::*;
    
    let results: Vec<DocumentParseResult> = paths.par_iter()
        .map(|path| parse_document(path.clone(), enable_ocr))
        .filter_map(|r| r.ok())
        .collect();
    
    Ok(results)
}

/// Check if a file extension is supported
#[napi]
pub fn is_supported_extension(path: String) -> bool {
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    matches!(
        ext.as_str(),
        "pdf" | "epub" | "html" | "htm" | "txt" | "md" | "markdown" | "rtf" |
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp"
    )
}

/// Get list of supported extensions
#[napi]
pub fn get_supported_extensions() -> Vec<String> {
    vec![
        ".pdf".to_string(),
        ".epub".to_string(),
        ".html".to_string(),
        ".htm".to_string(),
        ".txt".to_string(),
        ".md".to_string(),
        ".markdown".to_string(),
        ".rtf".to_string(),
        ".jpg".to_string(),
        ".jpeg".to_string(),
        ".png".to_string(),
        ".gif".to_string(),
        ".bmp".to_string(),
        ".tiff".to_string(),
        ".webp".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_supported_extension() {
        assert!(is_supported_extension("test.pdf".to_string()));
        assert!(is_supported_extension("test.html".to_string()));
        assert!(!is_supported_extension("test.xyz".to_string()));
    }
    
    #[test]
    fn test_get_supported_extensions() {
        let exts = get_supported_extensions();
        assert!(exts.contains(&".pdf".to_string()));
        assert!(exts.contains(&".html".to_string()));
    }
}
