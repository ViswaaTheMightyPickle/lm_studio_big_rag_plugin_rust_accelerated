//! Core indexing pipeline
//! Orchestrates the document indexing workflow:
//! 1. Scan directory
//! 2. Parse documents  
//! 3. Chunk texts
//! Note: Embedding and vector store indexing handled by TypeScript

use napi::bindgen_prelude::*;
use napi::Result as NapiResult;
use napi_derive::napi;
use rayon::prelude::*;
use std::path::Path;

use crate::chunking::{chunk_texts_batch, BatchChunkResult, TextChunk};

/// Parsed document with chunks
#[napi(object)]
pub struct ParsedDocumentWithChunks {
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub text: String,
    pub chunks: Vec<TextChunk>,
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

    // Filter chunks for file index 0 and convert
    let chunks: Vec<TextChunk> = chunk_results.into_iter()
        .filter(|c| c.file_index == 0)
        .map(|c| TextChunk {
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

/// Indexing statistics
#[napi(object)]
pub struct IndexingStats {
    pub total_files: u32,
    pub total_chunks: u32,
    pub total_tokens: u32,
    pub avg_chunks_per_file: f64,
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
