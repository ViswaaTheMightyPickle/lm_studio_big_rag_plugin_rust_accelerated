import * as path from "path";
import { type LMStudioClient } from "@lmstudio/sdk";
import { parseDocument as nativeParseDocument, isSupportedExtension, extractPdfText as nativeExtractPdfText } from "../native";

export interface ParsedDocument {
  text: string;
  metadata: {
    filePath: string;
    fileName: string;
    extension: string;
    parsedAt: Date;
  };
}

export type ParseFailureReason =
  | "unsupported-extension"
  | "pdf.missing-client"
  | "pdf.lmstudio-error"
  | "pdf.lmstudio-empty"
  | "pdf.pdfparse-error"
  | "pdf.pdfparse-empty"
  | "pdf.ocr-disabled"
  | "pdf.ocr-error"
  | "pdf.ocr-render-error"
  | "pdf.ocr-empty"
  | "epub.empty"
  | "html.empty"
  | "html.error"
  | "text.empty"
  | "text.error"
  | "image.ocr-disabled"
  | "image.empty"
  | "image.error"
  | "parser.unexpected-error";

export type DocumentParseResult =
  | { success: true; document: ParsedDocument }
  | { success: false; reason: ParseFailureReason; details?: string };

const MIN_TEXT_LENGTH = 50;

/**
 * Parse PDF using LM Studio parser (fallback for complex PDFs)
 */
async function parsePDFLMStudio(filePath: string, client: LMStudioClient): Promise<DocumentParseResult> {
  try {
    const fileHandle = await client.files.prepareFile(filePath);
    const result = await client.files.parseDocument(fileHandle);
    const text = result.content?.trim() ?? "";
    
    if (text.length >= MIN_TEXT_LENGTH) {
      return {
        success: true,
        document: {
          text,
          metadata: {
            filePath,
            fileName: path.basename(filePath),
            extension: ".pdf",
            parsedAt: new Date(),
          },
        },
      };
    }
    
    return {
      success: false,
      reason: "pdf.lmstudio-empty",
      details: `Extracted ${text.length} chars`,
    };
  } catch (error) {
    return {
      success: false,
      reason: "pdf.lmstudio-error",
      details: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Parse a document file using Rust native parser (primary) with LM Studio fallback for PDFs
 */
export async function parseDocument(
  filePath: string,
  enableOCR: boolean = false,
  client?: LMStudioClient,
): Promise<DocumentParseResult> {
  const ext = path.extname(filePath).toLowerCase();
  const fileName = path.basename(filePath);

  // Check if extension is supported
  if (!isSupportedExtension(filePath)) {
    return { 
      success: false, 
      reason: "unsupported-extension",
      details: ext
    };
  }

  try {
    // PDF: Use Rust parser first (fast), fallback to LM Studio for complex PDFs
    if (ext === ".pdf") {
      // Try Rust parser first (fastest)
      const rustResult = await nativeExtractPdfText(filePath);
      
      if (rustResult.success && rustResult.text.length >= MIN_TEXT_LENGTH) {
        return {
          success: true,
          document: {
            text: rustResult.text,
            metadata: {
              filePath,
              fileName,
              extension: ext,
              parsedAt: new Date(),
            },
          },
        };
      }
      
      // Fallback to LM Studio parser for complex PDFs
      if (!client) {
        return {
          success: false,
          reason: "pdf.missing-client",
          details: "LM Studio client required for PDF parsing fallback",
        };
      }
      
      console.log(`[Parser] Rust PDF extraction failed, trying LM Studio parser for ${fileName}`);
      const lmStudioResult = await parsePDFLMStudio(filePath, client);
      
      if (lmStudioResult.success) {
        return lmStudioResult;
      }
      
      return {
        success: false,
        reason: "pdf.pdfparse-empty",
        details: lmStudioResult.details || rustResult.error || "No text extracted",
      };
    }

    // Images: Use Rust OCR if enabled
    if ([".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".webp"].includes(ext)) {
      if (!enableOCR) {
        return {
          success: false,
          reason: "image.ocr-disabled",
          details: "Enable OCR to parse images",
        };
      }
      
      const { ocrImage } = await import("../native");
      const ocrResult = await ocrImage(filePath, {
        language: "eng",
        preprocessGrayscale: true,
        preprocessDeskew: false,
        enhanceContrast: true,
        maxImageArea: 50000000,
      });
      
      if (ocrResult.success && ocrResult.text.length >= MIN_TEXT_LENGTH) {
        return {
          success: true,
          document: {
            text: ocrResult.text,
            metadata: {
              filePath,
              fileName,
              extension: ext,
              parsedAt: new Date(),
            },
          },
        };
      }
      
      return {
        success: false,
        reason: ocrResult.success ? "image.empty" : "image.error",
        details: ocrResult.error || "No text extracted from image",
      };
    }

    // All other formats: Use Rust parser (primary)
    const result = await nativeParseDocument(filePath, enableOCR);

    if (result.success && result.text) {
      return {
        success: true,
        document: {
          text: result.text,
          metadata: {
            filePath: result.filePath,
            fileName: result.fileName,
            extension: result.extension,
            parsedAt: new Date(),
          },
        },
      };
    }

    // Map Rust error to TypeScript reason
    const reason = mapErrorToReason(result.error, ext, enableOCR);
    return {
      success: false,
      reason,
      details: result.error,
    };
  } catch (error) {
    console.error(`[Parser] Error parsing ${filePath}:`, error);
    return {
      success: false,
      reason: "parser.unexpected-error",
      details: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Map Rust error message to TypeScript ParseFailureReason
 */
function mapErrorToReason(
  error: string | undefined,
  ext: string,
  enableOCR: boolean,
): ParseFailureReason {
  if (!error) return "parser.unexpected-error";

  const err = error.toLowerCase();

  // OCR-related errors
  if (err.includes("ocr is disabled")) return "image.ocr-disabled";
  if (err.includes("ocr")) return "image.error";

  // File type specific errors
  if (ext === ".pdf") {
    if (err.includes("no text")) return "pdf.pdfparse-empty";
    return "pdf.pdfparse-error";
  }

  if (ext === ".epub") {
    if (err.includes("no text") || err.includes("empty")) return "epub.empty";
    return "text.error";
  }

  if (ext === ".html" || ext === ".htm") {
    if (err.includes("no text")) return "html.empty";
    return "html.error";
  }

  // Default
  if (err.includes("no text") || err.includes("empty")) {
    return "text.empty";
  }

  return "text.error";
}
