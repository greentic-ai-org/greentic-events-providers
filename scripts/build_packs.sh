#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

mkdir -p "${DIST_DIR}"

tar -czf "${DIST_DIR}/greentic-events-packs.tar.gz" \
  -C "${ROOT_DIR}" \
  packs \
  flows

echo "Pack artifact created at ${DIST_DIR}/greentic-events-packs.tar.gz"
