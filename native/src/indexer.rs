//! Complete indexing pipeline
//! Provides parse and chunk functionality with statistics

use napi::bindgen_prelude::*;
use napi::Result as NapiResult;
use napi_derive::napi;
use rayon::prelude::*;

/// Indexing statistics
#[napi(object)]
pub struct IndexingStats {
    pub total_files: u32,
    pub total_chunks: u32,
    pub total_tokens: u32,
    pub avg_chunks_per_file: f64,
}

/// Parsed document with chunks
#[napi(object)]
pub struct ParsedDocumentWithChunks {
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub text: String,
    pub chunks: Vec<crate::chunking::TextChunk>,
}

/// Parse and chunk a single document
#[napi]
pub fn parse_and_chunk_document(
    path: String,
    chunk_size: u32,
    chunk_overlap: u32,
    enable_ocr: bool,
) -> NapiResult<ParsedDocumentWithChunks> {
    // Parse document
    let parse_result = crate::parser::parse_document(path.clone(), enable_ocr)?;
    
    if !parse_result.success || parse_result.text.is_empty() {
        return Ok(ParsedDocumentWithChunks {
            file_path: parse_result.file_path,
            file_name: parse_result.file_name,
            extension: parse_result.extension,
            text: String::new(),
            chunks: Vec::new(),
        });
    }

    // Chunk the text
    let texts = vec![parse_result.text.clone()];
    let chunk_results = crate::chunking::chunk_texts_batch(texts, chunk_size, chunk_overlap)?;
    
    // Filter chunks for file index 0
    let chunks: Vec<crate::chunking::TextChunk> = chunk_results.into_iter()
        .filter(|c| c.file_index == 0)
        .map(|c| crate::chunking::TextChunk {
            text: c.text,
            start_index: c.start_index,
            end_index: c.end_index,
            token_estimate: c.token_estimate,
        })
        .collect();

    Ok(ParsedDocumentWithChunks {
        file_path: parse_result.file_path,
        file_name: parse_result.file_name,
        extension: parse_result.extension,
        text: parse_result.text,
        chunks,
    })
}

/// Parse and chunk multiple documents in parallel
#[napi]
pub fn parse_and_chunk_documents_batch(
    paths: Vec<String>,
    chunk_size: u32,
    chunk_overlap: u32,
    enable_ocr: bool,
) -> NapiResult<Vec<ParsedDocumentWithChunks>> {
    let results: Vec<ParsedDocumentWithChunks> = paths.par_iter()
        .map(|path| {
            parse_and_chunk_document(
                path.clone(),
                chunk_size,
                chunk_overlap,
                enable_ocr,
            ).unwrap_or(ParsedDocumentWithChunks {
                file_path: path.clone(),
                file_name: String::new(),
                extension: String::new(),
                text: String::new(),
                chunks: Vec::new(),
            })
        })
        .collect();

    Ok(results)
}

/// Calculate indexing statistics for a set of files
#[napi]
pub fn calculate_indexing_stats(
    paths: Vec<String>,
    chunk_size: u32,
    chunk_overlap: u32,
    enable_ocr: bool,
) -> NapiResult<IndexingStats> {
    let results = parse_and_chunk_documents_batch(paths, chunk_size, chunk_overlap, enable_ocr)?;
    
    let total_files = results.len() as u32;
    let total_chunks: u32 = results.iter().map(|r| r.chunks.len() as u32).sum();
    let total_tokens: u32 = results.iter()
        .flat_map(|r| r.chunks.iter())
        .map(|c| c.token_estimate)
        .sum();
    
    let avg_chunks = if total_files > 0 {
        total_chunks as f64 / total_files as f64
    } else {
        0.0
    };

    Ok(IndexingStats {
        total_files,
        total_chunks,
        total_tokens,
        avg_chunks_per_file: avg_chunks,
    })
}

/// Estimate token count for text (rough approximation)
#[napi]
pub fn estimate_tokens(text: String) -> u32 {
    // Rough estimate: 1 token ≈ 4 characters for English
    ((text.len() + 3) / 4) as u32
}

/// Batch estimate token counts
#[napi]
pub fn estimate_tokens_batch(texts: Vec<String>) -> Vec<u32> {
    texts.par_iter()
        .map(|text| ((text.len() + 3) / 4) as u32)
        .collect()
}
