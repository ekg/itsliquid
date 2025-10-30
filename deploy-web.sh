#!/bin/bash
set -e

echo "🔨 Building WASM..."
wasm-pack build --target web --out-dir web/pkg --release

echo "📤 Deploying to hypervolu.me..."
ssh hypervolu.me "mkdir -p www/itsliquid"
scp -r web/* hypervolu.me:www/itsliquid/

echo "✅ Deployed!"
echo "🌐 View at: https://hypervolu.me/~erik/itsliquid"
