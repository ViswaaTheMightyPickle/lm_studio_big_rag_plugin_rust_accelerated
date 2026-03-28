import * as fs from "fs";
import {
  parseTextFile,
  stripMarkdown as nativeStripMarkdown,
  normalizeText as nativeNormalizeText,
  isNativeAvailable,
} from "../native";

export interface ParseTextOptions {
  stripMarkdown?: boolean;
  preserveLineBreaks?: boolean;
}

/**
 * Parse plain text files (txt, md and related formats)
 * Uses Rust native implementation when available (4-6x faster for markdown stripping)
 */
export async function parseText(
  filePath: string,
  options: ParseTextOptions = {},
): Promise<string> {
  const { stripMarkdown = false, preserveLineBreaks = false } = options;

  // Use Rust native implementation when available
  if (isNativeAvailable() && parseTextFile) {
    const result = await parseTextFile(filePath, {
      strip_markdown: stripMarkdown,
      preserve_line_breaks: preserveLineBreaks,
      collapse_whitespace: true,
    });
    
    if (result.success) {
      return result.text;
    }
    
    // Fallback to TypeScript implementation on error
    console.warn(`[TextParser] Rust parser failed for ${filePath}, using fallback`);
  }

  // Fallback to TypeScript implementation
  try {
    const content = await fs.promises.readFile(filePath, "utf-8");
    const normalized = normalizeLineEndings(content);

    const stripped = stripMarkdown ? stripMarkdownSyntax(normalized) : normalized;

    return (preserveLineBreaks ? collapseWhitespaceButKeepLines(stripped) : collapseWhitespace(stripped)).trim();
  } catch (error) {
    console.error(`Error parsing text file ${filePath}:`, error);
    return "";
  }
}

function normalizeLineEndings(input: string): string {
  return input.replace(/\r\n?/g, "\n");
}

function collapseWhitespace(input: string): string {
  return input.replace(/\s+/g, " ");
}

function collapseWhitespaceButKeepLines(input: string): string {
  return (
    input
      // Trim trailing whitespace per line
      .replace(/[ \t]+\n/g, "\n")
      // Collapse multiple blank lines but keep paragraph separation
      .replace(/\n{3,}/g, "\n\n")
      // Collapse internal spaces/tabs
      .replace(/[ \t]{2,}/g, " ")
  );
}

function stripMarkdownSyntax(input: string): string {
  let output = input;

  // Remove fenced code blocks
  output = output.replace(/```[\s\S]*?```/g, " ");
  // Remove inline code
  output = output.replace(/`([^`]+)`/g, "$1");
  // Replace images with alt text
  output = output.replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1 ");
  // Replace links with link text
  output = output.replace(/\[([^\]]+)\]\([^)]*\)/g, "$1");
  // Remove emphasis markers
  output = output.replace(/(\*\*|__)(.*?)\1/g, "$2");
  output = output.replace(/(\*|_)(.*?)\1/g, "$2");
  // Remove headings
  output = output.replace(/^\s{0,3}#{1,6}\s+/gm, "");
  // Remove block quotes
  output = output.replace(/^\s{0,3}>\s?/gm, "");
  // Remove unordered list markers
  output = output.replace(/^\s{0,3}[-*+]\s+/gm, "");
  // Remove ordered list markers
  output = output.replace(/^\s{0,3}\d+[\.\)]\s+/gm, "");
  // Remove horizontal rules
  output = output.replace(/^\s{0,3}([-*_]\s?){3,}$/gm, "");
  // Remove residual HTML tags
  output = output.replace(/<[^>]+>/g, " ");

  return output;
}

/**
 * Strip markdown syntax using Rust native implementation (single-pass, 4-6x faster)
 */
export function stripMarkdown(input: string): string {
  if (isNativeAvailable() && nativeStripMarkdown) {
    return nativeStripMarkdown(input);
  }
  return stripMarkdownSyntax(input);
}

/**
 * Normalize text (line endings + whitespace) using Rust native implementation
 */
export function normalizeText(input: string, preserveLines: boolean = false): string {
  if (isNativeAvailable() && nativeNormalizeText) {
    return nativeNormalizeText(input, preserveLines);
  }
  
  const normalized = normalizeLineEndings(input);
  return preserveLines ? collapseWhitespaceButKeepLines(normalized) : collapseWhitespace(normalized);
}

