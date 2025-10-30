#!/bin/bash
# Automated browser testing for itsliquid WASM simulation
# This script builds the WASM, sets up dependencies, and runs E2E tests

set -e

echo "🧪 itsliquid WASM Browser Testing"
echo "=================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from project root directory"
    exit 1
fi

# Step 1: Build WASM
echo "📦 Step 1: Building WASM..."
wasm-pack build --target web --out-dir web/pkg --release
echo "✅ WASM build complete"
echo ""

# Step 2: Check if npm dependencies are installed
cd web
if [ ! -d "node_modules" ]; then
    echo "📥 Step 2: Installing npm dependencies..."
    npm install
    echo "✅ Dependencies installed"
else
    echo "✅ Step 2: Dependencies already installed"
fi
echo ""

# Step 3: Install browsers if needed
if [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo "🌐 Step 3: Installing browser binaries..."
    npx playwright install firefox chromium
    echo "✅ Browsers installed"
else
    echo "✅ Step 3: Browsers already installed"
fi
echo ""

# Step 4: Create test-results directory
mkdir -p test-results
echo "✅ Step 4: Test results directory ready"
echo ""

# Step 5: Run tests
echo "🚀 Step 5: Running E2E tests..."
echo ""

# Parse command line arguments
HEADED=""
PROJECT="firefox"
DEBUG=""
UI=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --headed)
            HEADED="--headed"
            shift
            ;;
        --chromium)
            PROJECT="chromium"
            shift
            ;;
        --firefox)
            PROJECT="firefox"
            shift
            ;;
        --debug)
            DEBUG="--debug"
            shift
            ;;
        --ui)
            UI="--ui"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--headed] [--firefox|--chromium] [--debug] [--ui]"
            exit 1
            ;;
    esac
done

# Build the test command
TEST_CMD="npx playwright test --project=$PROJECT $HEADED $DEBUG $UI"

echo "Running: $TEST_CMD"
echo ""

# Run the tests
if $TEST_CMD; then
    echo ""
    echo "✅ All tests passed!"
    echo ""
    echo "📊 Test artifacts:"
    echo "   - Screenshots: web/test-results/"
    echo "   - HTML report: Run 'cd web && npx playwright show-report' to view"
    echo ""
else
    echo ""
    echo "❌ Tests failed!"
    echo ""
    echo "📊 Debug information:"
    echo "   - Screenshots: web/test-results/"
    echo "   - HTML report: Run 'cd web && npx playwright show-report' to view"
    echo "   - Re-run with --debug flag for step-by-step debugging"
    echo ""
    exit 1
fi
