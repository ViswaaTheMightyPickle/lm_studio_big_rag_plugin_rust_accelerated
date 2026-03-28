//! Vector operations module for accelerated similarity search
//! Uses f64 for better precision

use napi::Result as NapiResult;
use napi_derive::napi;
use rayon::prelude::*;

/// Search result for vector similarity
#[napi(object)]
pub struct VectorSearchResult {
    pub index: u32,
    pub score: f64,
}

/// Compute cosine similarity between a query vector and multiple vectors
#[napi]
pub fn compute_cosine_similarities(vectors: Vec<Vec<f64>>, query: Vec<f64>) -> NapiResult<Vec<f64>> {
    if query.is_empty() {
        return Err(napi::Error::from_reason("Query vector cannot be empty"));
    }
    
    let query_norm = calculate_norm(&query);
    
    if query_norm == 0.0 {
        return Err(napi::Error::from_reason("Query vector has zero norm"));
    }
    
    let similarities: Vec<f64> = vectors.par_iter()
        .map(|vector| {
            if vector.is_empty() || vector.len() != query.len() {
                return 0.0;
            }
            
            let vec_norm = calculate_norm(vector);
            
            if vec_norm == 0.0 {
                return 0.0;
            }
            
            let dot = dot_product_raw(vector, &query);
            dot / (vec_norm * query_norm)
        })
        .collect();
    
    Ok(similarities)
}

/// Find top-k most similar vectors to query
#[napi]
pub fn find_top_k_similar(vectors: Vec<Vec<f64>>, query: Vec<f64>, k: u32) -> NapiResult<Vec<VectorSearchResult>> {
    if vectors.is_empty() || query.is_empty() {
        return Ok(Vec::new());
    }
    
    let similarities = compute_cosine_similarities(vectors, query)?;
    
    let mut indexed: Vec<(usize, f64)> = similarities.into_iter().enumerate().collect();
    indexed.par_sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let k = k as usize;
    let top_k: Vec<VectorSearchResult> = indexed.into_iter()
        .take(k)
        .map(|(index, score)| VectorSearchResult { index: index as u32, score })
        .collect();
    
    Ok(top_k)
}

/// Compute pairwise cosine similarity matrix
#[napi]
pub fn compute_similarity_matrix(vectors: Vec<Vec<f64>>) -> NapiResult<Vec<Vec<f64>>> {
    let n = vectors.len();
    if n == 0 { return Ok(Vec::new()); }
    
    let norms: Vec<f64> = vectors.par_iter().map(|v| calculate_norm(v)).collect();
    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    
    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            if norms[i] == 0.0 || norms[j] == 0.0 {
                matrix[i][j] = 0.0;
            } else {
                let dot = dot_product_raw(&vectors[i], &vectors[j]);
                let similarity = dot / (norms[i] * norms[j]);
                matrix[i][j] = similarity;
                matrix[j][i] = similarity;
            }
        }
    }
    
    Ok(matrix)
}

/// Normalize vectors to unit length (L2 normalization)
#[napi]
pub fn normalize_vectors(vectors: Vec<Vec<f64>>) -> NapiResult<Vec<Vec<f64>>> {
    let normalized: Vec<Vec<f64>> = vectors.into_par_iter()
        .map(|mut v| {
            let norm = calculate_norm(&v);
            if norm > 0.0 {
                for x in &mut v { *x /= norm; }
            }
            v
        })
        .collect();
    Ok(normalized)
}

/// Compute Euclidean distances between query and all vectors
#[napi]
pub fn compute_euclidean_distances(vectors: Vec<Vec<f64>>, query: Vec<f64>) -> NapiResult<Vec<f64>> {
    if vectors.is_empty() || query.is_empty() { return Ok(Vec::new()); }
    
    let distances: Vec<f64> = vectors.par_iter()
        .map(|vector| {
            if vector.len() != query.len() { return f64::MAX; }
            let sum_sq: f64 = vector.iter().zip(query.iter()).map(|(a, b)| (a - b).powi(2)).sum();
            sum_sq.sqrt()
        })
        .collect();
    
    Ok(distances)
}

/// Dot product between two vectors
#[napi]
pub fn dot_product(a: Vec<f64>, b: Vec<f64>) -> NapiResult<f64> {
    if a.len() != b.len() {
        return Err(napi::Error::from_reason("Vectors must have same length"));
    }
    Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
}

/// L2 norm (Euclidean length) of a vector
#[napi]
pub fn vector_norm(v: Vec<f64>) -> NapiResult<f64> {
    Ok(calculate_norm(&v))
}

/// Vector statistics
#[napi(object)]
pub struct VectorStats {
    pub dimension: u32,
    pub norm: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}

/// Get statistics for a vector
#[napi]
pub fn get_vector_stats(v: Vec<f64>) -> NapiResult<VectorStats> {
    if v.is_empty() {
        return Ok(VectorStats { dimension: 0, norm: 0.0, min: 0.0, max: 0.0, mean: 0.0 });
    }
    let norm = calculate_norm(&v);
    let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    Ok(VectorStats { dimension: v.len() as u32, norm, min, max, mean })
}

/// Batch get statistics for multiple vectors
#[napi]
pub fn get_vector_stats_batch(vectors: Vec<Vec<f64>>) -> NapiResult<Vec<VectorStats>> {
    let stats: Vec<VectorStats> = vectors.par_iter()
        .map(|v| {
            if v.is_empty() {
                return VectorStats { dimension: 0, norm: 0.0, min: 0.0, max: 0.0, mean: 0.0 };
            }
            let norm = calculate_norm(v);
            let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = v.iter().sum::<f64>() / v.len() as f64;
            VectorStats { dimension: v.len() as u32, norm, min, max, mean }
        })
        .collect();
    Ok(stats)
}

#[inline]
fn calculate_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

#[inline]
fn dot_product_raw(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let vectors = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]];
        let query = vec![1.0, 0.0, 0.0];
        let sims = compute_cosine_similarities(vectors, query).unwrap();
        assert!((sims[0] - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert_eq!(dot_product(a, b).unwrap(), 32.0);
    }
}
