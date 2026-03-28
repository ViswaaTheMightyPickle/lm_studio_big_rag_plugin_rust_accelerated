import * as path from "path";
import { type LMStudioClient } from "@lmstudio/sdk";
import { parseDocument as nativeParseDocument, isSupportedExtension } from "../native";

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

/**
 * Parse a document file using Rust native parser
 * All parsing is done in Rust for maximum performance
 */
export async function parseDocument(
  filePath: string,
  enableOCR: boolean = false,
  _client?: LMStudioClient,  // Kept for API compatibility
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
    // Call Rust native parser
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
