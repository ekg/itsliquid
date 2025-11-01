#!/bin/bash
set -e

echo "🔨 Building WASM..."
wasm-pack build --target web --out-dir web/pkg --release

echo "📤 Deploying to hypervolu.me..."
ssh hypervolu.me "mkdir -p www/itsliquid"
# Only deploy what's needed (exclude node_modules, test files, etc.)
rsync -avz --exclude 'node_modules' --exclude 'test-results' --exclude 'playwright-report' --exclude 'package*.json' --exclude 'playwright.config.js' --exclude 'tests' --exclude '.gitignore' --exclude 'TESTING.md' web/ hypervolu.me:www/itsliquid/

echo "✅ Deployed!"
echo "🌐 View at: https://hypervolu.me/~erik/itsliquid"
