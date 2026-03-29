import { type LMStudioClient, type EmbeddingDynamicHandle } from "@lmstudio/sdk";
import { IndexManager, type IndexingProgress, type IndexingResult } from "./indexManager";
import { VectorStore } from "../vectorstore/vectorStore";

export interface RunIndexingParams {
  client: LMStudioClient;
  abortSignal: AbortSignal;
  documentsDir: string;
  vectorStoreDir: string;
  chunkSize: number;
  chunkOverlap: number;
  maxConcurrent: number;
  enableOCR: boolean;
  autoReindex: boolean;
  parseDelayMs: number;
  forceReindex?: boolean;
  vectorStore?: VectorStore;
  onProgress?: (progress: IndexingProgress) => void;
  // Embedding parallelization settings
  embeddingBatchSize?: number;
  embeddingConcurrency?: number;
  embeddingModelId?: string;
}

export interface RunIndexingResult {
  summary: string;
  stats: {
    totalChunks: number;
    uniqueFiles: number;
  };
  indexingResult: IndexingResult;
}

/**
 * Shared helper that runs the full indexing pipeline.
 * Uses a single embedding model with configurable batch size and concurrency.
 */
export async function runIndexingJob({
  client,
  abortSignal,
  documentsDir,
  vectorStoreDir,
  chunkSize,
  chunkOverlap,
  maxConcurrent,
  enableOCR,
  autoReindex,
  parseDelayMs,
  forceReindex = false,
  vectorStore: existingVectorStore,
  onProgress,
  embeddingBatchSize = 20,       // Smaller batches = faster completion, less timeout risk
  embeddingConcurrency = 4,      // Lower concurrency = more stable over VPN/Tailscale
  embeddingModelId = "nomic-ai/nomic-embed-text-v1.5-GGUF",
}: RunIndexingParams): Promise<RunIndexingResult> {
  const vectorStore = existingVectorStore ?? new VectorStore(vectorStoreDir);
  const ownsVectorStore = existingVectorStore === undefined;

  if (ownsVectorStore) {
    await vectorStore.initialize();
  }

  // Load single embedding model
  console.log(`[BigRAG] Loading embedding model: ${embeddingModelId}`);
  
  const embeddingModels: EmbeddingDynamicHandle[] = [];

  try {
    // Check if model is already loaded
    const loadedModels = await client.embedding.listLoaded();
    console.log(`[BigRAG] Found ${loadedModels.length} already loaded embedding model(s)`);

    if (loadedModels.length > 0) {
      // Use the first loaded model
      embeddingModels.push(loadedModels[0]);
      console.log(`[BigRAG] Using already loaded model: ${loadedModels[0]}`);
    } else {
      // Load the model
      console.log(`[BigRAG] Loading embedding model...`);
      await client.embedding.load(embeddingModelId);
      const model = await client.embedding.model(embeddingModelId);
      embeddingModels.push(model);
      console.log(`[BigRAG] Successfully loaded embedding model`);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    console.error('[BigRAG] Failed to load embedding model:', errorMsg);
    console.error('[BigRAG] Make sure:');
    console.error('[BigRAG]   1. LM Studio is running');
    console.error('[BigRAG]   2. An embedding model is downloaded');
    console.error(`[BigRAG]   3. Run "lms get ${embeddingModelId}" to download one`);
    throw new Error(`Failed to load embedding model: ${errorMsg}`);
  }

  console.log(`[BigRAG] Using 1 embedding model with batch size ${embeddingBatchSize}, concurrency ${embeddingConcurrency}`);

  const indexManager = new IndexManager({
    documentsDir,
    vectorStore,
    vectorStoreDir,
    embeddingModels,
    client,
    chunkSize,
    chunkOverlap,
    maxConcurrent,
    enableOCR,
    autoReindex: forceReindex ? false : autoReindex,
    parseDelayMs,
    abortSignal,
    onProgress,
    embeddingBatchSize,
    embeddingConcurrency,
  });

  const indexingResult = await indexManager.index();
  const stats = await vectorStore.getStats();

  if (ownsVectorStore) {
    await vectorStore.close();
  }

  const summary = `Indexing completed!\n\n` +
    `• Successfully indexed: ${indexingResult.successfulFiles}/${indexingResult.totalFiles}\n` +
    `• Failed: ${indexingResult.failedFiles}\n` +
    `• Skipped (unchanged): ${indexingResult.skippedFiles}\n` +
    `• Updated existing files: ${indexingResult.updatedFiles}\n` +
    `• New files added: ${indexingResult.newFiles}\n` +
    `• Chunks in store: ${stats.totalChunks}\n` +
    `• Unique files in store: ${stats.uniqueFiles}`;

  return {
    summary,
    stats,
    indexingResult,
  };
}
