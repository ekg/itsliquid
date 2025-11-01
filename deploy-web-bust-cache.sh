#!/bin/bash
set -e

# Generate a new version timestamp
VERSION=$(date +"%Y.%m.%d.%H%M")

echo "🔨 Building WASM with version ${VERSION}..."
wasm-pack build --target web --out-dir web/pkg --release

echo "🔄 Updating version in HTML..."
# Update version in JavaScript
sed -i "s/const VERSION = '[^']*'/const VERSION = '${VERSION}'/" web/index.html
# Update version in footer
sed -i "s/v[0-9]\+\.[0-9]\+\.[0-9]\+\.[0-9]\+/v${VERSION}/" web/index.html

echo "📤 Deploying to hypervolu.me..."
ssh hypervolu.me "mkdir -p www/itsliquid"
rsync -avz --exclude 'node_modules' --exclude 'test-results' --exclude 'playwright-report' --exclude 'package*.json' --exclude 'playwright.config.js' --exclude 'tests' --exclude '.gitignore' --exclude 'TESTING.md' web/ hypervolu.me:www/itsliquid/

echo "✅ Deployed version ${VERSION}!"
echo "🌐 View at: https://hypervolu.me/~erik/itsliquid"
echo ""
echo "⚠️  Users need to HARD REFRESH to see changes:"
echo "   Desktop: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)"
echo "   Mobile: Clear browser cache or use incognito mode"
