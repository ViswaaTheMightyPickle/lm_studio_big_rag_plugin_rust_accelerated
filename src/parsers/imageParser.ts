import { ocrImage as nativeOcrImage, isNativeAvailable } from "../native";

/**
 * Parse image files using OCR
 * Uses Rust native implementation when available (8-15x faster with parallel processing)
 */
export async function parseImage(filePath: string): Promise<string> {
  // Use Rust native implementation when available
  if (isNativeAvailable() && nativeOcrImage) {
    try {
      const result = await nativeOcrImage(filePath, {
        language: "eng",
        preprocess_grayscale: true,
        preprocess_deskew: false,
        enhance_contrast: true,
        max_image_area: 50_000_000,
      });
      
      if (result.success && result.text) {
        console.log(`[Image Parser] (Rust OCR) Extracted ${result.text.length} chars from ${filePath}`);
        return result.text.trim();
      }
      
      // Fallback to TypeScript implementation on error
      console.warn(`[Image Parser] Rust OCR failed for ${filePath}, using fallback: ${result.error}`);
    } catch (error) {
      console.warn(`[Image Parser] Rust OCR error for ${filePath}, using fallback:`, error);
    }
  }

  // Fallback to TypeScript implementation (Tesseract.js)
  return parseImageTypeScript(filePath);
}

/**
 * TypeScript fallback implementation using Tesseract.js
 */
async function parseImageTypeScript(filePath: string): Promise<string> {
  try {
    const { createWorker } = await import("tesseract.js");
    const worker = await createWorker("eng");

    const { data: { text } } = await worker.recognize(filePath);

    await worker.terminate();

    return text
      .replace(/\s+/g, " ")
      .replace(/\n+/g, "\n")
      .trim();
  } catch (error) {
    console.error(`Error parsing image file ${filePath}:`, error);
    return "";
  }
}

