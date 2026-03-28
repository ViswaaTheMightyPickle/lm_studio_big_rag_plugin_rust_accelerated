# Embedding Benchmark Results

**Date:** 2026-03-28T20:48:06.368Z
**Test Chunks:** 500
**Model:** nomic-ai/nomic-embed-text-v1.5

## Results

| Configuration | Models | Batch Size | Concurrency | Time (s) | Chunks/sec | Success |
|--------------|--------|------------|-------------|----------|------------|---------|
| Single Model, Batch 100, Concurrency 3 | 1 | 100 | 3 | 14.80 | 34 | ✅
| Single Model, Batch 250, Concurrency 5 | 1 | 250 | 5 | 15.73 | 32 | ✅
| Single Model, Batch 500, Concurrency 10 | 1 | 500 | 10 | 16.20 | 31 | ✅
| 2 Models, Batch 100, Concurrency 3 | 2 | 100 | 3 | 16.74 | 30 | ✅
| 2 Models, Batch 250, Concurrency 5 | 2 | 250 | 5 | 16.50 | 30 | ✅
| 2 Models, Batch 500, Concurrency 10 | 2 | 500 | 10 | 16.65 | 30 | ✅

## Best Results


**Fastest:** Single Model, Batch 100, Concurrency 3 - 14.80s

**Highest Throughput:** Single Model, Batch 100, Concurrency 3 - 34 chunks/sec


## Raw Data

```json
[
  {
    "configName": "Single Model, Batch 100, Concurrency 3",
    "modelCount": 1,
    "batchSize": 100,
    "concurrency": 3,
    "totalTimeMs": 14797,
    "chunksPerSecond": 34,
    "success": true
  },
  {
    "configName": "Single Model, Batch 250, Concurrency 5",
    "modelCount": 1,
    "batchSize": 250,
    "concurrency": 5,
    "totalTimeMs": 15734,
    "chunksPerSecond": 32,
    "success": true
  },
  {
    "configName": "Single Model, Batch 500, Concurrency 10",
    "modelCount": 1,
    "batchSize": 500,
    "concurrency": 10,
    "totalTimeMs": 16196,
    "chunksPerSecond": 31,
    "success": true
  },
  {
    "configName": "2 Models, Batch 100, Concurrency 3",
    "modelCount": 2,
    "batchSize": 100,
    "concurrency": 3,
    "totalTimeMs": 16741,
    "chunksPerSecond": 30,
    "success": true
  },
  {
    "configName": "2 Models, Batch 250, Concurrency 5",
    "modelCount": 2,
    "batchSize": 250,
    "concurrency": 5,
    "totalTimeMs": 16501,
    "chunksPerSecond": 30,
    "success": true
  },
  {
    "configName": "2 Models, Batch 500, Concurrency 10",
    "modelCount": 2,
    "batchSize": 500,
    "concurrency": 10,
    "totalTimeMs": 16653,
    "chunksPerSecond": 30,
    "success": true
  }
]
```
