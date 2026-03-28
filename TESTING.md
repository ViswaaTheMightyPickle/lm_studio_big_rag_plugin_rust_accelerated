# BigRAG Plugin - Testing Guide

## Test Suite Overview

The BigRAG plugin includes comprehensive tests for all Rust native modules.

## Running Tests

### Full Test Suite

```bash
# Run all Rust parser tests with real data
node testRustParsers.cjs /path/to/documents

# Example with test data
node testRustParsers.cjs /home/pickle/Storage/RAG_Pipeline_Docs
```

### Rust Unit Tests

```bash
cd native
cargo test
```

### TypeScript Integration Tests

```bash
npm run test
npm run test:all
```

## Test Coverage

### Rust Native Modules (22 tests)

| Category | Tests | Status |
|----------|-------|--------|
| Directory Scanner | 3 | ✅ |
| Text Parser | 3 | ✅ |
| HTML Parser | 2 | ✅ |
| Tokenizer | 3 | ✅ |
| Text Chunking | 2 | ✅ |
| File Hashing | 2 | ✅ |
| OCR | 2 | ✅ |
| Vector Operations | 5 | ✅ |
| Document Router | 1 | ✅ |

### Expected Output

```
=== BigRAG Rust Native Module Tests ===

✅ Native module loaded successfully

=== Directory Scanner Tests ===
🧪 Testing: scanDirectory - basic functionality
   Found 7422 files
   Sample file: example.pdf
✅ PASSED

=== Text Parser Tests ===
🧪 Testing: parseTextFile - first text file
   Testing with: README.md
   Time: 0ms
   Success: true
   Text length: 715 chars
✅ PASSED

...

=== Test Summary ===
Total: 22 tests
✅ Passed: 22
❌ Failed: 0
⚠️  Skipped: 0
```

## Test Data

### Using Provided Test Data

```bash
# If you have the test dataset
node testRustParsers.cjs /home/pickle/Storage/RAG_Pipeline_Docs
```

### Creating Test Data

```bash
# Generate test documents
node benchmarks/generateDataset.js 100

# This creates 100 test files in benchmark-data/
```

## Performance Testing

### Benchmark Suite

```bash
# Run all benchmarks
npm run bench

# Individual benchmarks
npm run bench:hash
npm run bench:chunk
npm run bench:scan
npm run bench:embedding
```

### Expected Performance

| Operation | Expected Time |
|-----------|--------------|
| Directory Scan (7000+ files) | <100ms |
| File Hashing (single) | <1ms |
| Text Parsing (1KB) | <1ms |
| HTML Parsing (10KB) | <5ms |
| PDF Extraction (10 pages) | 20-400ms |
| Text Chunking (10K words) | <10ms |

## Debugging

### Enable Debug Output

```bash
# Verbose test output
RUST_LOG=debug node testRustParsers.cjs /path/to/docs

# Check native module load errors
node -e "const n = require('./native'); console.log(n.getNativeLoadError())"
```

### Common Issues

**Native module not loading:**
```bash
# Rebuild native module
cd native && npm run build

# Check for missing dependencies
ldd native/*.node
```

**OCR tests failing:**
```bash
# Verify tesseract installation
tesseract --version

# Check if OCR is available
node -e "const n = require('./native'); console.log(n.getOcrLanguages())"
```

## CI/CD Testing

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install dependencies
        run: npm install
      
      - name: Build native module
        run: cd native && npm run build
      
      - name: Run tests
        run: node testRustParsers.cjs ./test-data
```

## Writing New Tests

### Rust Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example() {
        let result = some_function();
        assert!(result.success);
    }
}
```

### TypeScript Integration Tests

```javascript
const native = require('./native');

// Test function
const result = native.someFunction('input');
console.assert(result.success, 'Should succeed');
```

## Performance Regression Testing

```bash
# Run benchmarks and save results
npm run bench > benchmarks/results-$(date +%Y%m%d).txt

# Compare with previous results
diff benchmarks/results-previous.txt benchmarks/results-current.txt
```

## Contact

For test failures or issues, open a GitHub issue with:
- Test output
- System information
- Rust/Node.js versions
- Steps to reproduce
