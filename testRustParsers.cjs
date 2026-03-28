#!/usr/bin/env node
/**
 * Rust Native Module Test Script
 * Tests all Rust parser implementations with real data
 * 
 * Usage: node testRustParsers.cjs [documents_dir]
 */

const path = require('path');
const fs = require('fs');

// Load native module
const native = require('./native');

console.log('=== BigRAG Rust Native Module Tests ===\n');

// Check if native module is available
if (!native.isNativeAvailable()) {
  console.error('❌ Native module not available!');
  console.error('Error:', native.getNativeLoadError());
  process.exit(1);
}

console.log('✅ Native module loaded successfully\n');

// Test directory (can be overridden via command line)
const TEST_DIR = process.argv[2] || process.env.BIG_RAG_TEST_DIR || './test-documents';

// Test results tracking
const results = {
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: []
};

function test(name, fn) {
  try {
    console.log(`🧪 Testing: ${name}`);
    fn();
    console.log(`✅ PASSED: ${name}\n`);
    results.passed++;
    results.tests.push({ name, status: 'passed' });
  } catch (error) {
    console.error(`❌ FAILED: ${name}`);
    console.error(`   Error: ${error.message}\n`);
    results.failed++;
    results.tests.push({ name, status: 'failed', error: error.message });
  }
}

async function testAsync(name, fn) {
  try {
    console.log(`🧪 Testing: ${name}`);
    await fn();
    console.log(`✅ PASSED: ${name}\n`);
    results.passed++;
    results.tests.push({ name, status: 'passed' });
  } catch (error) {
    console.error(`❌ FAILED: ${name}`);
    console.error(`   Error: ${error.message}\n`);
    results.failed++;
    results.tests.push({ name, status: 'failed', error: error.message });
  }
}

// Find test files
function findTestFiles(dir, extensions) {
  const files = [];
  
  function walk(currentDir) {
    try {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile()) {
          const ext = path.extname(entry.name).toLowerCase();
          if (extensions.includes(ext)) {
            files.push(fullPath);
          }
        }
      }
    } catch (err) {
      // Ignore inaccessible directories
    }
  }
  
  walk(dir);
  return files;
}

// ============ SCANNER TESTS ============
console.log('=== Directory Scanner Tests ===\n');

test('scanDirectory - basic functionality', () => {
  const result = native.scanDirectory(TEST_DIR);
  if (!Array.isArray(result)) {
    throw new Error('Expected array result');
  }
  console.log(`   Found ${result.length} files`);
  if (result.length === 0) {
    console.log(`   ⚠️  No files found in ${TEST_DIR}`);
  } else {
    console.log(`   Sample file: ${result[0].name}`);
  }
});

test('getSupportedExtensions', () => {
  const exts = native.getSupportedExtensions();
  if (!Array.isArray(exts) || exts.length === 0) {
    throw new Error('Expected non-empty array');
  }
  console.log(`   Supported extensions: ${exts.join(', ')}`);
});

test('isSupportedExtension', () => {
  const supported = native.isSupportedExtension('test.pdf');
  const unsupported = native.isSupportedExtension('test.xyz');
  if (!supported) throw new Error('PDF should be supported');
  if (unsupported) throw new Error('XYZ should not be supported');
  console.log('   PDF supported: ✓, XYZ unsupported: ✓');
});

// ============ PDF PARSER TESTS ============
console.log('=== PDF Parser Tests ===\n');

const pdfFiles = findTestFiles(TEST_DIR, ['.pdf']);
if (pdfFiles.length > 0) {
  testAsync('extractPdfText - first PDF', async () => {
    const pdfPath = pdfFiles[0];
    console.log(`   Testing with: ${path.basename(pdfPath)}`);
    
    const startTime = Date.now();
    const result = native.extractPdfText(pdfPath);
    const elapsed = Date.now() - startTime;
    
    console.log(`   Time: ${elapsed}ms`);
    console.log(`   Success: ${result.success}`);
    console.log(`   Stage: ${result.stage}`);
    console.log(`   Text length: ${result.text.length} chars`);
    
    if (!result.success) {
      console.log(`   Error: ${result.error}`);
    } else if (result.text.length < 10) {
      throw new Error('Extracted text too short');
    }
  });
} else {
  test('extractPdfText - SKIPPED (no PDF files)', () => {
    results.skipped++;
  });
}

// ============ TEXT PARSER TESTS ============
console.log('=== Text Parser Tests ===\n');

const txtFiles = findTestFiles(TEST_DIR, ['.txt', '.md']);
if (txtFiles.length > 0) {
  test('parseTextFile - first text file', () => {
    const txtPath = txtFiles[0];
    console.log(`   Testing with: ${path.basename(txtPath)}`);
    
    const startTime = Date.now();
    const result = native.parseTextFile(txtPath, {
      stripMarkdown: txtPath.endsWith('.md'),  // camelCase for JS
      preserveLineBreaks: false,
      collapseWhitespace: true
    });
    const elapsed = Date.now() - startTime;
    
    console.log(`   Time: ${elapsed}ms`);
    console.log(`   Success: ${result.success}`);
    console.log(`   Text length: ${result.text.length} chars`);
    console.log(`   Word count: ${result.wordCount || result.word_count}`);
    
    if (!result.success) {
      throw new Error(`Parse failed: ${result.error}`);
    }
  });
  
  test('stripMarkdown', () => {
    const markdown = '# Hello\n\nThis is **bold** and *italic*.\n\n- Item 1\n- Item 2';
    const result = native.stripMarkdown(markdown);
    console.log(`   Input: "${markdown}"`);
    console.log(`   Output: "${result}"`);
    
    if (result.includes('#') || result.includes('**')) {
      throw new Error('Markdown not stripped properly');
    }
  });
  
  test('normalizeText', () => {
    const text = 'Hello\r\nWorld\rTest';
    const result = native.normalizeText(text, false);
    console.log(`   Input: "${text}"`);
    console.log(`   Output: "${result}"`);
    
    if (result.includes('\r')) {
      throw new Error('Line endings not normalized');
    }
  });
} else {
  test('parseTextFile - SKIPPED (no text files)', () => {
    results.skipped++;
  });
}

// ============ HTML PARSER TESTS ============
console.log('=== HTML Parser Tests ===\n');

const htmlFiles = findTestFiles(TEST_DIR, ['.html', '.htm']);
if (htmlFiles.length > 0) {
  testAsync('parseHtml - first HTML file', async () => {
    const htmlPath = htmlFiles[0];
    console.log(`   Testing with: ${path.basename(htmlPath)}`);
    
    const startTime = Date.now();
    const result = native.parseHtml(htmlPath);
    const elapsed = Date.now() - startTime;
    
    console.log(`   Time: ${elapsed}ms`);
    console.log(`   Success: ${result.success}`);
    console.log(`   Text length: ${result.char_count} chars`);
    
    if (!result.success) {
      console.log(`   Error: ${result.error}`);
    }
  });
} else {
  test('parseHtml - SKIPPED (no HTML files)', () => {
    results.skipped++;
  });
}

test('parseHtmlString', () => {
  const html = '<html><body><h1>Hello</h1><p>World</p></body></html>';
  const result = native.parseHtmlString(html);
  console.log(`   Input: "${html}"`);
  console.log(`   Output: "${result}"`);
  
  if (!result.includes('Hello') || !result.includes('World')) {
    throw new Error('HTML parsing failed');
  }
});

// ============ TOKENIZER TESTS ============
console.log('=== Tokenizer Tests ===\n');

test('countTokens', () => {
  const text = 'Hello, this is a test sentence for token counting.';
  const count = native.countTokens(text);
  console.log(`   Text: "${text}"`);
  console.log(`   Token count: ${count}`);
  
  if (count <= 0) {
    throw new Error('Token count should be positive');
  }
});

test('validateTokenLimit', () => {
  const shortText = 'Short text';
  const longText = 'A'.repeat(10000);
  
  const shortValid = native.validateTokenLimit(shortText, 100);
  const longValid = native.validateTokenLimit(longText, 100);
  
  console.log(`   Short text (within limit): ${shortValid ? '✓' : '✗'}`);
  console.log(`   Long text (exceeds limit): ${!longValid ? '✓' : '✗'}`);
  
  if (!shortValid || longValid) {
    throw new Error('Token validation failed');
  }
});

test('chunkByTokens', () => {
  const text = 'This is a test. '.repeat(100);
  const chunks = native.chunkByTokens(text, 50, 10);
  console.log(`   Input length: ${text.length} chars`);
  console.log(`   Number of chunks: ${chunks.length}`);
  console.log(`   First chunk tokens: ${chunks[0]?.token_count}`);
  
  if (chunks.length === 0) {
    throw new Error('No chunks produced');
  }
});

// ============ CHUNKING TESTS ============
console.log('=== Text Chunking Tests ===\n');

test('chunkText', () => {
  const text = 'Hello world. This is a test. '.repeat(50);
  const chunks = native.chunkText(text, 10, 2);
  console.log(`   Input length: ${text.length} chars`);
  console.log(`   Number of chunks: ${chunks.length}`);
  console.log(`   First chunk: "${chunks[0]?.text?.substring(0, 50)}..."`);
  
  if (chunks.length === 0) {
    throw new Error('No chunks produced');
  }
});

test('chunkTextFast', () => {
  const text = 'Hello world. This is a test. '.repeat(50);
  const chunks = native.chunkTextFast(text, 10, 2);
  console.log(`   Input length: ${text.length} chars`);
  console.log(`   Number of chunks: ${chunks.length}`);
  
  if (!Array.isArray(chunks) || chunks.length === 0) {
    throw new Error('Expected array of chunks');
  }
});

// ============ HASHING TESTS ============
console.log('=== File Hashing Tests ===\n');

if (txtFiles.length > 0) {
  test('hashFile', () => {
    const filePath = txtFiles[0];
    console.log(`   Testing with: ${path.basename(filePath)}`);
    
    const startTime = Date.now();
    const hash = native.hashFile(filePath);
    const elapsed = Date.now() - startTime;
    
    console.log(`   Time: ${elapsed}ms`);
    console.log(`   Hash: ${hash.substring(0, 16)}...`);
    
    if (!hash || hash.length !== 64) {
      throw new Error('Invalid hash format');
    }
  });
  
  test('hashFilesParallel', () => {
    const filesToHash = txtFiles.slice(0, 5);
    console.log(`   Hashing ${filesToHash.length} files in parallel`);
    
    const startTime = Date.now();
    const results = native.hashFilesParallel(filesToHash);
    const elapsed = Date.now() - startTime;
    
    console.log(`   Time: ${elapsed}ms`);
    console.log(`   Results: ${results.length} hashes`);
    
    if (results.length !== filesToHash.length) {
      throw new Error('Hash count mismatch');
    }
  });
} else {
  test('hashFile - SKIPPED (no files)', () => {
    results.skipped++;
  });
}

// ============ OCR TESTS ============
console.log('=== OCR Tests ===\n');

const imageFiles = findTestFiles(TEST_DIR, ['.png', '.jpg', '.jpeg']);
if (imageFiles.length > 0) {
  test('ocrImage', () => {
    const imagePath = imageFiles[0];
    console.log(`   Testing with: ${path.basename(imagePath)}`);
    
    const result = native.ocrImage(imagePath, {
      language: 'eng',
      preprocessGrayscale: true,  // camelCase for JS
      preprocessDeskew: false,
      enhanceContrast: true,
      maxImageArea: 50000000
    });
    
    console.log(`   Success: ${result.success}`);
    console.log(`   Text length: ${result.text.length} chars`);
    
    if (!result.success) {
      console.log(`   Note: ${result.error}`);
    }
  });
} else {
  test('ocrImage - SKIPPED (no image files)', () => {
    results.skipped++;
  });
}

test('getOcrLanguages', () => {
  const languages = native.getOcrLanguages();
  console.log(`   Available languages: ${languages.join(', ')}`);
  
  if (!languages.includes('eng')) {
    throw new Error('English should be available');
  }
});

// ============ VECTOR OPERATIONS TESTS ============
console.log('=== Vector Operations Tests ===\n');

test('computeCosineSimilarities', () => {
  const vectors = [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1]
  ];
  const query = [1, 0, 0];
  
  const similarities = native.computeCosineSimilarities(vectors, query);
  console.log(`   Query: [1, 0, 0]`);
  console.log(`   Similarities: ${similarities.map(s => s.toFixed(2)).join(', ')}`);
  
  if (Math.abs(similarities[0] - 1.0) > 0.001) {
    throw new Error('First vector should have similarity 1.0');
  }
});

test('findTopKSimilar', () => {
  const vectors = [
    [1, 0, 0],
    [0.9, 0.1, 0],
    [0, 1, 0],
    [0, 0, 1]
  ];
  const query = [1, 0, 0];
  
  const results = native.findTopKSimilar(vectors, query, 2);
  console.log(`   Top 2 similar:`);
  results.forEach((r, i) => {
    console.log(`     ${i + 1}. Index ${r.index}, Score: ${r.score.toFixed(3)}`);
  });
  
  if (results.length !== 2) {
    throw new Error('Should return 2 results');
  }
  if (results[0].index !== 0) {
    throw new Error('First result should be index 0');
  }
});

test('normalizeVectors', () => {
  const vectors = [[3, 4], [6, 8]];
  const normalized = native.normalizeVectors(vectors);
  console.log(`   Input: [[3, 4], [6, 8]]`);
  console.log(`   Output: [[${normalized[0].map(n => n.toFixed(2)).join(', ')}], [${normalized[1].map(n => n.toFixed(2)).join(', ')}]]`);
  
  // Check that norms are 1.0
  const norm0 = Math.sqrt(normalized[0][0] ** 2 + normalized[0][1] ** 2);
  if (Math.abs(norm0 - 1.0) > 0.001) {
    throw new Error('Normalized vector should have norm 1.0');
  }
});

test('dotProduct', () => {
  const a = [1, 2, 3];
  const b = [4, 5, 6];
  const result = native.dotProduct(a, b);
  console.log(`   [1, 2, 3] · [4, 5, 6] = ${result}`);
  
  if (result !== 32) {
    throw new Error(`Expected 32, got ${result}`);
  }
});

test('vectorNorm', () => {
  const v = [3, 4];
  const norm = native.vectorNorm(v);
  console.log(`   ||[3, 4]|| = ${norm}`);
  
  if (Math.abs(norm - 5.0) > 0.001) {
    throw new Error(`Expected 5.0, got ${norm}`);
  }
});

// ============ DOCUMENT ROUTER TESTS ============
console.log('=== Document Router Tests ===\n');

if (txtFiles.length > 0) {
  test('parseDocument - text file', () => {
    const filePath = txtFiles[0];
    console.log(`   Testing with: ${path.basename(filePath)}`);
    
    const result = native.parseDocument(filePath, false);
    console.log(`   Success: ${result.success}`);
    console.log(`   Extension: ${result.extension}`);
    console.log(`   Text length: ${result.text.length} chars`);
    
    if (!result.success && result.error?.includes('TypeScript fallback')) {
      console.log('   ℹ️  Using TypeScript fallback (expected for some types)');
    }
  });
}

// ============ SUMMARY ============
console.log('\n=== Test Summary ===\n');
console.log(`Total: ${results.passed + results.failed + results.skipped} tests`);
console.log(`✅ Passed: ${results.passed}`);
console.log(`❌ Failed: ${results.failed}`);
console.log(`⚠️  Skipped: ${results.skipped}`);

if (results.failed > 0) {
  console.log('\nFailed tests:');
  results.tests
    .filter(t => t.status === 'failed')
    .forEach(t => console.log(`  - ${t.name}: ${t.error}`));
}

process.exit(results.failed > 0 ? 1 : 0);
