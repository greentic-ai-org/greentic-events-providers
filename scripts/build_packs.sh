#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist/packs"
PACKC_VERSION="${PACKC_VERSION:-0.4}"

PACKC_BIN="$(command -v packc || true)"
if [ -z "${PACKC_BIN}" ]; then
  echo "packc not found. Install with: cargo install packc --version ${PACKC_VERSION} --locked" >&2
  exit 1
fi

INSTALLED_PACKC_VERSION="$(${PACKC_BIN} --version | awk '{print $2}')"
if [[ "${INSTALLED_PACKC_VERSION}" != "${PACKC_VERSION}" && "${INSTALLED_PACKC_VERSION}" != ${PACKC_VERSION}.* ]]; then
  echo "packc ${PACKC_VERSION}.x required (found ${INSTALLED_PACKC_VERSION}). Install with: cargo install packc --version ${PACKC_VERSION} --locked --force" >&2
  exit 1
fi

# Ensure wasm32-wasip2 target is available for the active toolchain, even though
# packc builds happen in a temp dir outside this repo (and thus outside the
# rust-toolchain override).
ACTIVE_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-$(rustup show active-toolchain 2>/dev/null | cut -d' ' -f1)}"
if [ -z "${ACTIVE_TOOLCHAIN}" ]; then
  ACTIVE_TOOLCHAIN="$(rustup default | awk '{print $1}')"
fi

rustup target add --toolchain "${ACTIVE_TOOLCHAIN}" wasm32-wasip2 >/dev/null 2>&1 || true
if ! rustup target list --toolchain "${ACTIVE_TOOLCHAIN}" --installed | grep -q "wasm32-wasip2"; then
  echo "Rust target wasm32-wasip2 not installed for toolchain ${ACTIVE_TOOLCHAIN}. Run: rustup target add --toolchain ${ACTIVE_TOOLCHAIN} wasm32-wasip2" >&2
  exit 1
fi

# Force packc/cargo invocations (in /tmp) to use the same toolchain.
export RUSTUP_TOOLCHAIN="${ACTIVE_TOOLCHAIN}"

export PACKC_LOG=warn
export CARGO_TERM_PROGRESS_WHEN=never

if [ "${PACKC_DEBUG:-0}" != 0 ]; then
  echo "Using toolchain: ${ACTIVE_TOOLCHAIN}"
  rustc +"${ACTIVE_TOOLCHAIN}" --version
  echo "Installed targets for ${ACTIVE_TOOLCHAIN}:"
  rustup target list --toolchain "${ACTIVE_TOOLCHAIN}" --installed
fi

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "${TMP_ROOT}"' EXIT

for pack in "${ROOT_DIR}"/packs/events/*.yaml; do
  name="$(basename "${pack%.*}")"
  out="${DIST_DIR}/${name}.gtpack"
  work_dir="${TMP_ROOT}/${name}"
  mkdir -p "${work_dir}"

  # copy manifest as pack.yaml and supporting assets
  cp "${pack}" "${work_dir}/pack.yaml"
  if [ -d "${ROOT_DIR}/flows" ]; then
    rsync -a "${ROOT_DIR}/flows" "${work_dir}/"
  fi
  if [ -d "${ROOT_DIR}/packs/components" ]; then
    rsync -a "${ROOT_DIR}/packs/components/" "${work_dir}/components/"
  fi

  echo "Building pack: ${pack} -> ${out}"
  "${PACKC_BIN}" build \
    --log warn \
    --in "${work_dir}" \
    --gtpack-out "${out}" \
    --manifest "${DIST_DIR}/${name}.cbor" \
    --sbom "${DIST_DIR}/${name}.sbom.json"
done

echo "Pack artifacts created under ${DIST_DIR}"
