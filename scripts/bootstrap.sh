#!/usr/bin/env bash
# Local preview: clone Quartz into .build/, overlay content + config,
# install deps, and serve at http://localhost:8080 with live reload.
set -euo pipefail

QUARTZ_VERSION="${QUARTZ_VERSION:-v4}"
BUILD_DIR=".build"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

if [ ! -d "$BUILD_DIR/.git" ]; then
  echo "→ Cloning Quartz $QUARTZ_VERSION into $BUILD_DIR/"
  rm -rf "$BUILD_DIR"
  git clone --depth 1 --branch "$QUARTZ_VERSION" \
    https://github.com/jackyzha0/quartz.git "$BUILD_DIR"
fi

echo "→ Overlaying content/ and config"
rm -rf "$BUILD_DIR/content"
cp -r content "$BUILD_DIR/content"
cp quartz.config.ts "$BUILD_DIR/quartz.config.ts"
cp quartz.layout.ts "$BUILD_DIR/quartz.layout.ts"

cd "$BUILD_DIR"
if [ ! -d node_modules ]; then
  echo "→ Installing dependencies"
  npm install
fi

echo "→ Serving at http://localhost:8080"
npx quartz build --serve
