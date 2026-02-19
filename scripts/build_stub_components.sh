#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if ! command -v wasm-tools >/dev/null 2>&1; then
  echo "wasm-tools is required but was not found on PATH." >&2
  exit 1
fi

CRATE_MANIFEST="${ROOT_DIR}/components/stub-component-v060/Cargo.toml"
TARGET_DIR="${ROOT_DIR}/target"
OUT_DIR="${ROOT_DIR}/packs/components/stubs"

ACTIVE_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-$(rustup show active-toolchain 2>/dev/null | cut -d' ' -f1)}"
if [ -z "${ACTIVE_TOOLCHAIN}" ]; then
  ACTIVE_TOOLCHAIN="$(rustup default | awk '{print $1}')"
fi
rustup target add --toolchain "${ACTIVE_TOOLCHAIN}" wasm32-unknown-unknown >/dev/null 2>&1 || true

mkdir -p "${OUT_DIR}"

component_ids=(
  "stub"
  "templating.handlebars"
  "events-email-source"
  "events-email-sink"
  "events-sms-source"
  "events-sms-sink"
  "events-timer-source"
  "events-webhook-source"
)

component_version_for_id() {
  case "$1" in
    stub) echo "0.1.0" ;;
    templating.handlebars) echo "0.1.0" ;;
    events-email-source|events-email-sink|events-sms-source|events-sms-sink|events-timer-source|events-webhook-source)
      echo "1.0.0"
      ;;
    *)
      echo "Unknown stub component id: $1" >&2
      exit 1
      ;;
  esac
}

for component_id in "${component_ids[@]}"; do
  component_version="$(component_version_for_id "${component_id}")"
  echo "Building stub component for id=${component_id}"
  STUB_COMPONENT_ID="${component_id}" STUB_COMPONENT_VERSION="${component_version}" cargo +"${ACTIVE_TOOLCHAIN}" build \
    --offline \
    --release \
    --target wasm32-unknown-unknown \
    --manifest-path "${CRATE_MANIFEST}"

  core_wasm="${TARGET_DIR}/wasm32-unknown-unknown/release/stub_component_v060.wasm"
  out_wasm="${OUT_DIR}/${component_id}.wasm"
  wasm-tools component new "${core_wasm}" -o "${out_wasm}"
done

cp "${OUT_DIR}/stub.wasm" "${ROOT_DIR}/packs/components/stub.wasm"
cp "${OUT_DIR}/templating.handlebars.wasm" "${ROOT_DIR}/packs/components/templating.handlebars/stub.wasm"

echo "Stub components generated under ${OUT_DIR}"
