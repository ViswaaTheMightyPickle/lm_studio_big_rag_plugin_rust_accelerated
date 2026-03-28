//! Image OCR module
//! Full OCR requires tesseract crate which needs system dependencies:
//! sudo dnf install tesseract-ocr libleptonica-dev clang-devel
//! 
//! Uses tesseract for OCR and image crate for preprocessing

use napi::Result as NapiResult;
use napi_derive::napi;
use rayon::prelude::*;
use image::{DynamicImage, GenericImageView, imageops};
use std::path::Path;

/// OCR result for a single image
#[napi(object)]
pub struct OcrImageResult {
    pub path: String,
    pub text: String,
    pub success: bool,
    pub error: Option<String>,
}

/// OCR configuration options
#[napi(object)]
#[derive(Clone)]
pub struct OcrConfig {
    #[napi(js_name = "language")]
    pub language: String,
    #[napi(js_name = "preprocessGrayscale")]
    pub preprocess_grayscale: bool,
    #[napi(js_name = "preprocessDeskew")]
    pub preprocess_deskew: bool,
    #[napi(js_name = "enhanceContrast")]
    pub enhance_contrast: bool,
    #[napi(js_name = "maxImageArea")]
    pub max_image_area: f64,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            preprocess_grayscale: true,
            preprocess_deskew: false,
            enhance_contrast: true,
            max_image_area: 50_000_000.0,
        }
    }
}

/// Perform OCR on a single image file
#[napi]
pub fn ocr_image(path: String, config: Option<OcrConfig>) -> NapiResult<OcrImageResult> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Ok(OcrImageResult {
            path: path.to_string_lossy().to_string(),
            text: String::new(),
            success: false,
            error: Some("File does not exist".to_string()),
        });
    }
    
    let config = config.unwrap_or_default();
    
    match image::open(path) {
        Ok(img) => {
            let (width, height) = img.dimensions();
            let area = (width as f64) * (height as f64);
            
            if area > config.max_image_area {
                return Ok(OcrImageResult {
                    path: path.to_string_lossy().to_string(),
                    text: String::new(),
                    success: false,
                    error: Some(format!(
                        "Image too large: {}x{} (max: {} pixels)",
                        width, height, config.max_image_area as u32
                    )),
                });
            }
            
            let processed = preprocess_image(&img, &config);
            
            match perform_ocr(&processed, &config.language) {
                Ok(text) => Ok(OcrImageResult {
                    path: path.to_string_lossy().to_string(),
                    text,
                    success: true,
                    error: None,
                }),
                Err(e) => Ok(OcrImageResult {
                    path: path.to_string_lossy().to_string(),
                    text: String::new(),
                    success: false,
                    error: Some(format!("OCR failed: {}", e)),
                }),
            }
        }
        Err(e) => Ok(OcrImageResult {
            path: path.to_string_lossy().to_string(),
            text: String::new(),
            success: false,
            error: Some(format!("Failed to load image: {}", e)),
        }),
    }
}

/// Perform OCR on multiple images in parallel
#[napi]
pub fn ocr_images_batch(paths: Vec<String>, config: Option<OcrConfig>) -> NapiResult<Vec<OcrImageResult>> {
    let config = config.unwrap_or_default();
    
    let results: Vec<OcrImageResult> = paths.par_iter()
        .map(|path| {
            let path = Path::new(path);
            
            if !path.exists() {
                return OcrImageResult {
                    path: path.to_string_lossy().to_string(),
                    text: String::new(),
                    success: false,
                    error: Some("File does not exist".to_string()),
                };
            }
            
            match image::open(path) {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    let area = (width as f64) * (height as f64);
                    
                    if area > config.max_image_area {
                        return OcrImageResult {
                            path: path.to_string_lossy().to_string(),
                            text: String::new(),
                            success: false,
                            error: Some(format!("Image too large: {}x{}", width, height)),
                        };
                    }
                    
                    let processed = preprocess_image(&img, &config);
                    
                    match perform_ocr(&processed, &config.language) {
                        Ok(text) => OcrImageResult {
                            path: path.to_string_lossy().to_string(),
                            text,
                            success: true,
                            error: None,
                        },
                        Err(e) => OcrImageResult {
                            path: path.to_string_lossy().to_string(),
                            text: String::new(),
                            success: false,
                            error: Some(format!("OCR failed: {}", e)),
                        },
                    }
                }
                Err(e) => OcrImageResult {
                    path: path.to_string_lossy().to_string(),
                    text: String::new(),
                    success: false,
                    error: Some(format!("Failed to load image: {}", e)),
                },
            }
        })
        .collect();
    
    Ok(results)
}

/// Preprocess image for better OCR accuracy
fn preprocess_image(image: &DynamicImage, config: &OcrConfig) -> DynamicImage {
    let mut result = image.clone();

    if config.preprocess_grayscale {
        result = result.grayscale();
    }

    if config.enhance_contrast {
        result = imageops::contrast(&result, 30.0).into();
    }

    result
}

/// Perform OCR using tesseract
fn perform_ocr(image: &DynamicImage, lang: &str) -> Result<String, String> {
    use tesseract::{Tesseract, PageSegMode};
    use image::ImageFormat;
    
    // Convert image to PNG in memory
    let mut buffer = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;
    
    // Initialize tesseract and chain operations
    let mut tess = Tesseract::new(None, Some(lang))
        .map_err(|e| format!("Failed to initialize tesseract: {}", e))?;
    
    tess.set_page_seg_mode(PageSegMode::PsmAuto);
    
    let tess = tess.set_image_from_mem(&buffer)
        .map_err(|e| format!("Failed to set image: {}", e))?;
    
    let mut tess = tess.recognize()
        .map_err(|e| format!("OCR recognition failed: {}", e))?;
    
    let text = tess.get_text()
        .map_err(|e| format!("Failed to get text: {}", e))?;
    
    Ok(text.trim().to_string())
}

/// Get OCR statistics for an image
#[napi(object)]
pub struct OcrStats {
    pub width: u32,
    pub height: u32,
    pub area: f64,
    pub estimated_processing_time_ms: f64,
}

/// Get statistics about an image for OCR planning
#[napi]
pub fn get_image_ocr_stats(path: String) -> NapiResult<OcrStats> {
    let path = Path::new(&path);
    
    if !path.exists() {
        return Ok(OcrStats {
            width: 0,
            height: 0,
            area: 0.0,
            estimated_processing_time_ms: 0.0,
        });
    }
    
    match image::open(path) {
        Ok(img) => {
            let (width, height) = img.dimensions();
            let area = (width as f64) * (height as f64);
            let estimated_time_ms = area / 1000.0;
            
            Ok(OcrStats {
                width,
                height,
                area,
                estimated_processing_time_ms: estimated_time_ms,
            })
        }
        Err(_) => Ok(OcrStats {
            width: 0,
            height: 0,
            area: 0.0,
            estimated_processing_time_ms: 0.0,
        }),
    }
}

/// Supported OCR languages
#[napi]
pub fn get_ocr_languages() -> Vec<String> {
    vec![
        "eng".to_string(),
        "spa".to_string(),
        "fra".to_string(),
        "deu".to_string(),
        "ita".to_string(),
        "por".to_string(),
        "rus".to_string(),
        "chi_sim".to_string(),
        "chi_tra".to_string(),
        "jpn".to_string(),
        "kor".to_string(),
        "ara".to_string(),
        "hin".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ocr_image_nonexistent() {
        let result = ocr_image("nonexistent.png".to_string(), None).unwrap();
        assert!(!result.success);
    }
    
    #[test]
    fn test_get_ocr_languages() {
        let languages = get_ocr_languages();
        assert!(languages.contains(&"eng".to_string()));
        assert!(!languages.is_empty());
    }
}
