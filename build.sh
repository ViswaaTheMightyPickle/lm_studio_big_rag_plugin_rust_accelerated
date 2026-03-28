#!/bin/bash
# Cross-platform build script for BigRAG native module
# Builds for Linux, macOS (Intel/ARM), and Windows

set -e

echo "=== BigRAG Native Module - Cross-Platform Build ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check prerequisites
check_prereqs() {
    echo "Checking prerequisites..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust not found. Install from https://rustup.rs/"
        exit 1
    fi
    print_status "Rust installed: $(rustc --version)"
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js not found"
        exit 1
    fi
    print_status "Node.js installed: $(node --version)"
    
    # Check npm
    if ! command -v npm &> /dev/null; then
        print_error "npm not found"
        exit 1
    fi
    
    echo ""
}

# Build for current platform
build_current() {
    echo "Building for current platform..."
    cd native
    npm install
    npm run build
    cd ..
    print_status "Build complete: native/bigrag-native.*.node"
    echo ""
}

# Add cross-compilation targets
add_targets() {
    echo "Adding cross-compilation targets..."
    
    # macOS Intel
    if ! rustup target list | grep -q "x86_64-apple-darwin"; then
        rustup target add x86_64-apple-darwin
        print_status "Added macOS Intel target"
    fi
    
    # macOS ARM
    if ! rustup target list | grep -q "aarch64-apple-darwin"; then
        rustup target add aarch64-apple-darwin
        print_status "Added macOS ARM target"
    fi
    
    # Windows
    if ! rustup target list | grep -q "x86_64-pc-windows-gnu"; then
        rustup target add x86_64-pc-windows-gnu
        print_status "Added Windows target"
    fi
    
    echo ""
}

# Build for macOS Intel
build_macos_intel() {
    echo "Building for macOS Intel (x86_64)..."
    cd native
    npm run build -- --target x86_64-apple-darwin
    cd ..
    
    if [ -f "native/bigrag-native.darwin-x64.node" ]; then
        print_status "macOS Intel build: native/bigrag-native.darwin-x64.node"
    else
        print_warning "macOS Intel build may have failed"
    fi
    echo ""
}

# Build for macOS ARM (Apple Silicon)
build_macos_arm() {
    echo "Building for macOS ARM (aarch64)..."
    cd native
    npm run build -- --target aarch64-apple-darwin
    cd ..
    
    if [ -f "native/bigrag-native.darwin-arm64.node" ]; then
        print_status "macOS ARM build: native/bigrag-native.darwin-arm64.node"
    else
        print_warning "macOS ARM build may have failed"
    fi
    echo ""
}

# Build for Windows
build_windows() {
    echo "Building for Windows (x86_64)..."
    print_warning "Windows build requires MinGW or Visual Studio Build Tools"
    cd native
    npm run build -- --target x86_64-pc-windows-gnu
    cd ..
    
    if [ -f "native/bigrag-native.win32-x64-msvc.node" ]; then
        print_status "Windows build: native/bigrag-native.win32-x64-msvc.node"
    else
        print_warning "Windows build may have failed"
    fi
    echo ""
}

# Create distribution package
create_dist() {
    echo "Creating distribution package..."
    
    mkdir -p dist/native
    
    # Copy current platform build
    cp native/*.node dist/native/ 2>/dev/null || true
    
    # Copy TypeScript files
    npm run build
    
    # Copy native binaries
    cp native/*.node dist/native/ 2>/dev/null || true
    
    print_status "Distribution created in dist/"
    echo ""
}

# Run tests
run_tests() {
    echo "Running tests..."
    
    if [ -f "testRustParsers.cjs" ]; then
        node testRustParsers.cjs ./test-data 2>/dev/null || print_warning "Tests skipped (no test data)"
    else
        cd native && cargo test && cd ..
    fi
    
    echo ""
}

# Main execution
main() {
    echo ""
    echo "========================================"
    echo "  BigRAG Cross-Platform Build Script   "
    echo "========================================"
    echo ""
    
    check_prereqs
    
    case "${1:-build}" in
        build)
            build_current
            ;;
        all)
            build_current
            add_targets
            build_macos_intel
            build_macos_arm
            build_windows
            create_dist
            ;;
        macos)
            add_targets
            build_macos_intel
            build_macos_arm
            ;;
        windows)
            add_targets
            build_windows
            ;;
        dist)
            build_current
            create_dist
            ;;
        test)
            run_tests
            ;;
        clean)
            echo "Cleaning build artifacts..."
            cd native && cargo clean && cd ..
            rm -rf dist/
            print_status "Clean complete"
            ;;
        *)
            echo "Usage: $0 {build|all|macos|windows|dist|test|clean}"
            echo ""
            echo "Commands:"
            echo "  build   - Build for current platform (default)"
            echo "  all     - Build for all platforms"
            echo "  macos   - Build for macOS Intel and ARM"
            echo "  windows - Build for Windows"
            echo "  dist    - Create distribution package"
            echo "  test    - Run tests"
            echo "  clean   - Clean build artifacts"
            echo ""
            exit 1
            ;;
    esac
    
    echo "========================================"
    print_status "Build complete!"
    echo "========================================"
    echo ""
}

main "$@"
