//! HTML parser module
//! Uses scraper for proper HTML parsing (faster and more accurate than regex)

use napi::Result as NapiResult;
use napi_derive::napi;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;

/// HTML parsing result
#[napi(object)]
pub struct HtmlParseResult {
    pub success: bool,
    pub text: String,
    pub char_count: u32,
    pub error: Option<String>,
}

/// Parse HTML file and extract text content
#[napi]
pub fn parse_html(path: String) -> NapiResult<HtmlParseResult> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Ok(HtmlParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            error: Some("File does not exist".to_string()),
        });
    }
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let document = Html::parse_document(&content);
            
            // Extract text from body or all text nodes
            let text: String = document.root_element().text().collect();
            
            // Clean up whitespace
            let cleaned: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");
            let char_count = cleaned.len() as u32;
            let is_empty = cleaned.is_empty();
            
            Ok(HtmlParseResult {
                success: !is_empty,
                text: cleaned,
                char_count,
                error: if is_empty {
                    Some("No text content found in HTML".to_string())
                } else {
                    None
                },
            })
        }
        Err(e) => Ok(HtmlParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            error: Some(format!("Failed to read file: {}", e)),
        }),
    }
}

/// Parse HTML string and extract text content
#[napi]
pub fn parse_html_string(html: String) -> NapiResult<String> {
    let document = Html::parse_document(&html);
    let text: String = document.root_element().text().collect();
    Ok(text.split_whitespace().collect::<Vec<&str>>().join(" "))
}

/// Extract text from HTML with specific CSS selectors
#[napi]
pub fn parse_html_with_selectors(path: String, selectors: Vec<String>) -> NapiResult<HtmlParseResult> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Ok(HtmlParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            error: Some("File does not exist".to_string()),
        });
    }
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let document = Html::parse_document(&content);
            let mut text_parts = Vec::new();
            
            for selector_str in selectors {
                if let Ok(selector) = Selector::parse(&selector_str) {
                    for element in document.select(&selector) {
                        let text: String = element.text().collect();
                        if !text.trim().is_empty() {
                            text_parts.push(text.trim().to_string());
                        }
                    }
                }
            }
            
            let cleaned: String = text_parts.join(" ");
            let char_count = cleaned.len() as u32;
            let is_empty = cleaned.is_empty();
            
            Ok(HtmlParseResult {
                success: !is_empty,
                text: cleaned,
                char_count,
                error: if is_empty {
                    Some("No text content found matching selectors".to_string())
                } else {
                    None
                },
            })
        }
        Err(e) => Ok(HtmlParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            error: Some(format!("Failed to read file: {}", e)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_html_string() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let result = parse_html_string(html.to_string()).unwrap();
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }
}
