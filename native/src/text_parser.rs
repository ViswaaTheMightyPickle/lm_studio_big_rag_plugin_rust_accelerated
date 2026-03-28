//! Text and markdown parser module
//! Uses pulldown-cmark for proper markdown parsing (single-pass, faster than regex)

use napi::Result as NapiResult;
use napi_derive::napi;
use pulldown_cmark::{Parser, Options, Event, Tag, CodeBlockKind};
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Text parsing options
#[napi(object)]
pub struct TextParseOptions {
    #[napi(js_name = "stripMarkdown")]
    pub strip_markdown: bool,
    #[napi(js_name = "preserveLineBreaks")]
    pub preserve_line_breaks: bool,
    #[napi(js_name = "collapseWhitespace")]
    pub collapse_whitespace: bool,
}

impl Default for TextParseOptions {
    fn default() -> Self {
        Self {
            strip_markdown: false,
            preserve_line_breaks: false,
            collapse_whitespace: true,
        }
    }
}

/// Text parsing result
#[napi(object)]
pub struct TextParseResult {
    pub success: bool,
    pub text: String,
    #[napi(js_name = "charCount")]
    pub char_count: u32,
    #[napi(js_name = "wordCount")]
    pub word_count: u32,
    pub error: Option<String>,
}

/// Parse text file with optional markdown stripping
#[napi]
pub fn parse_text_file(path: String, options: Option<TextParseOptions>) -> NapiResult<TextParseResult> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Ok(TextParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            word_count: 0,
            error: Some("File does not exist".to_string()),
        });
    }
    
    let options = options.unwrap_or_default();
    
    match fs::read_to_string(path) {
        Ok(content) => {
            let text = process_text(&content, &options);
            
            Ok(TextParseResult {
                success: true,
                text: text.clone(),
                char_count: text.len() as u32,
                word_count: text.split_whitespace().count() as u32,
                error: None,
            })
        }
        Err(e) => Ok(TextParseResult {
            success: false,
            text: String::new(),
            char_count: 0,
            word_count: 0,
            error: Some(format!("Failed to read file: {}", e)),
        }),
    }
}

/// Process text with given options
fn process_text(text: &str, options: &TextParseOptions) -> String {
    let processed = if options.strip_markdown {
        strip_markdown(text.to_string())
    } else {
        text.to_string()
    };

    let normalized = normalize_line_endings(&processed);

    if options.preserve_line_breaks {
        if options.collapse_whitespace {
            collapse_whitespace_keep_lines(&normalized)
        } else {
            normalized
        }
    } else if options.collapse_whitespace {
        collapse_whitespace(&normalized)
    } else {
        normalized
    }
}

/// Strip markdown syntax using pulldown-cmark (single-pass parsing)
#[napi]
pub fn strip_markdown(text: String) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&text, options);

    let mut result = String::new();

    for event in parser {
        match event {
            Event::Text(text) => {
                result.push_str(&text);
            }
            Event::Code(code) => {
                // Inline code - keep the content
                result.push_str(&code);
            }
            Event::SoftBreak | Event::HardBreak => {
                result.push(' ');
            }
            Event::Rule => {
                // Horizontal rule - skip
            }
            Event::Start(tag) => {
                // Handle start tags - most are skipped
                match tag {
                    Tag::CodeBlock(CodeBlockKind::Fenced(info)) => {
                        // Fenced code block - we could preserve the info string
                        let _ = info;
                    }
                    Tag::CodeBlock(CodeBlockKind::Indented) => {
                        // Indented code block - process as text
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                // End tags - skip
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                // HTML tags - skip
            }
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {
                // Footnotes and task markers - skip
            }
        }
    }

    result
}

/// Normalize line endings to Unix style (\n)
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace("\r", "\n")
}

/// Collapse all whitespace to single spaces
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Collapse whitespace but preserve line breaks
fn collapse_whitespace_keep_lines(text: &str) -> String {
    text.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join("\n")
}

/// Parse multiple text files in parallel
#[napi]
pub fn parse_text_files_batch(
    paths: Vec<String>,
    options: Option<TextParseOptions>,
) -> NapiResult<Vec<TextParseResult>> {
    let options = options.unwrap_or_default();
    
    let results: Vec<TextParseResult> = paths.par_iter()
        .map(|path| {
            let path = Path::new(path);
            
            if !path.exists() {
                return TextParseResult {
                    success: false,
                    text: String::new(),
                    char_count: 0,
                    word_count: 0,
                    error: Some("File does not exist".to_string()),
                };
            }
            
            match fs::read_to_string(path) {
                Ok(content) => {
                    let text = process_text(&content, &options);
                    
                    TextParseResult {
                        success: true,
                        text: text.clone(),
                        char_count: text.len() as u32,
                        word_count: text.split_whitespace().count() as u32,
                        error: None,
                    }
                }
                Err(e) => TextParseResult {
                    success: false,
                    text: String::new(),
                    char_count: 0,
                    word_count: 0,
                    error: Some(format!("Failed to read file: {}", e)),
                },
            }
        })
        .collect();
    
    Ok(results)
}

/// Strip markdown from multiple texts in parallel
#[napi]
pub fn strip_markdown_batch(texts: Vec<String>) -> Vec<String> {
    texts.into_par_iter()
        .map(|text| strip_markdown(text))
        .collect()
}

/// Normalize text (line endings + whitespace)
#[napi]
pub fn normalize_text(text: String, preserve_lines: bool) -> String {
    let normalized = normalize_line_endings(&text);
    
    if preserve_lines {
        collapse_whitespace_keep_lines(&normalized)
    } else {
        collapse_whitespace(&normalized)
    }
}

/// Get text statistics
#[napi(object)]
pub struct TextStats {
    pub char_count: u32,
    pub word_count: u32,
    pub line_count: u32,
    pub avg_word_length: f64,
}

/// Get statistics for text
#[napi]
pub fn get_text_stats(text: String) -> TextStats {
    let char_count = text.len() as u32;
    let words: Vec<&str> = text.split_whitespace().collect();
    let word_count = words.len() as u32;
    let line_count = text.lines().count() as u32;
    
    let avg_word_length = if word_count > 0 {
        words.iter().map(|w| w.len()).sum::<usize>() as f64 / word_count as f64
    } else {
        0.0
    };
    
    TextStats {
        char_count,
        word_count,
        line_count,
        avg_word_length,
    }
}

/// Get text statistics for multiple texts in parallel
#[napi]
pub fn get_text_stats_batch(texts: Vec<String>) -> Vec<TextStats> {
    texts.par_iter()
        .map(|text| get_text_stats(text.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strip_markdown() {
        let markdown = "# Hello\n\nThis is **bold** and *italic*.\n\n- List item\n- Another\n\n```\ncode block\n```";
        let result = strip_markdown(markdown);
        
        assert!(result.contains("Hello"));
        assert!(result.contains("bold"));
        assert!(result.contains("italic"));
        assert!(!result.contains("**"));
        assert!(!result.contains('*'));
        assert!(!result.contains('#'));
    }
    
    #[test]
    fn test_normalize_line_endings() {
        let text = "Line 1\r\nLine 2\rLine 3\nLine 4";
        let result = normalize_line_endings(text);
        assert_eq!(result, "Line 1\nLine 2\nLine 3\nLine 4");
    }
    
    #[test]
    fn test_collapse_whitespace() {
        let text = "Hello    world\n\ttest";
        let result = collapse_whitespace(text);
        assert_eq!(result, "Hello world test");
    }
}
