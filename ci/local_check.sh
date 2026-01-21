#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

echo "==> cargo fmt"
cargo fmt --all -- --check

echo "==> cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo test"
cargo test --workspace

echo "==> build packs"
bash scripts/build_packs.sh

echo "==> greentic-pack doctor"
mkdir -p dist/packs
packs=(dist/events-*.gtpack)
if [ "${#packs[@]}" -eq 0 ]; then
  echo "No pack artifacts found for doctor validation." >&2
  exit 1
fi
for pack in "${packs[@]}"; do
  cp "${pack}" "dist/packs/$(basename "${pack}")"
done
if ls dist/packs/events-*.gtpack >/dev/null 2>&1; then
  for pack in dist/packs/events-*.gtpack; do
    greentic-pack doctor --validate --pack "${pack}"
  done
else
  echo "No pack artifacts found for doctor validation." >&2
  exit 1
fi

echo "==> greentic-provision conformance"
bash scripts/provision_conformance.sh

echo "All checks passed."
