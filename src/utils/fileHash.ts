import * as fs from "fs";
import * as crypto from "crypto";
import { hashFile, hashFilesParallel, isNativeAvailable } from "../native";

/**
 * Calculate SHA-256 hash of a file for change detection
 * Uses Rust native implementation when available (memory-mapped I/O, 1.5-4.5x faster)
 * Falls back to Node.js crypto module (OpenSSL-backed, highly optimized C code)
 */
export async function calculateFileHash(filePath: string): Promise<string> {
  // Use Rust native implementation when available
  if (isNativeAvailable() && hashFile) {
    return await hashFile(filePath);
  }
  
  // Fallback to Node.js crypto
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash("sha256");
    const stream = fs.createReadStream(filePath);

    stream.on("data", (data) => hash.update(data));
    stream.on("end", () => resolve(hash.digest("hex")));
    stream.on("error", reject);
  });
}

/**
 * Calculate SHA-256 hash of multiple files in parallel
 * Uses Rust native batch hashing when available (1.5-4.5x faster)
 * Falls back to Node.js crypto with Promise.all for parallel execution
 */
export async function calculateFileHashesParallel(filePaths: string[]): Promise<Map<string, string>> {
  // Use Rust native batch hashing when available
  if (isNativeAvailable() && hashFilesParallel) {
    const results = await hashFilesParallel(filePaths);
    return new Map(results.map(r => [r.path, r.hash!]));
  }
  
  // Fallback to Node.js crypto
  const hashPromises = filePaths.map(async (filePath) => {
    const hash = await calculateFileHash(filePath);
    return [filePath, hash] as [string, string];
  });

  const results = await Promise.all(hashPromises);
  return new Map(results);
}

/**
 * Get file metadata including size and modification time
 */
export async function getFileMetadata(filePath: string): Promise<{
  size: number;
  mtime: Date;
  hash: string;
}> {
  const stats = await fs.promises.stat(filePath);
  const hash = await calculateFileHash(filePath);

  return {
    size: stats.size,
    mtime: stats.mtime,
    hash,
  };
}

