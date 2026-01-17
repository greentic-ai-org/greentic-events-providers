#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
PACK_INSTALL_CMD=${PACK_INSTALL_CMD:-cargo binstall greentic-pack --locked}

PACK_BIN="${PACK_BIN:-$(command -v greentic-pack || true)}"
if [ -z "${PACK_BIN}" ]; then
  echo "greentic-pack not found. Install with: ${PACK_INSTALL_CMD}" >&2
  exit 1
fi

INSTALLED_PACK_VERSION="$(${PACK_BIN} --version | awk '{print $2}')"

# Optional: allow callers to enforce a major.minor series (e.g., PACK_SERIES=0.4.)
if [ -n "${PACK_SERIES:-}" ]; then
  if [[ "${INSTALLED_PACK_VERSION}" != "${PACK_SERIES}"* ]]; then
    echo "greentic-pack ${PACK_SERIES%?} required (found ${INSTALLED_PACK_VERSION}). Install with: ${PACK_INSTALL_CMD}" >&2
    exit 1
  fi
fi

# Ensure wasm32-wasip2 target is available for the active toolchain, even though
# greentic-pack builds happen in a temp dir outside this repo (and thus outside the
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

# Force greentic-pack/cargo invocations (in /tmp) to use the same toolchain.
export RUSTUP_TOOLCHAIN="${ACTIVE_TOOLCHAIN}"

export GREENTIC_PACK_LOG=warn
export CARGO_TERM_PROGRESS_WHEN=never

if [ "${PACK_DEBUG:-0}" != 0 ]; then
  echo "Using toolchain: ${ACTIVE_TOOLCHAIN}"
  rustc +"${ACTIVE_TOOLCHAIN}" --version
  echo "Installed targets for ${ACTIVE_TOOLCHAIN}:"
  rustup target list --toolchain "${ACTIVE_TOOLCHAIN}" --installed
fi

mkdir -p "${DIST_DIR}"
find "${DIST_DIR}" -maxdepth 1 -type f -name 'events-*.gtpack' -delete
find "${DIST_DIR}" -maxdepth 1 -type f -name 'events-*.cbor' -delete
find "${DIST_DIR}" -maxdepth 1 -type f -name 'events-*.sbom.json' -delete

PACK_ROOT="${ROOT_DIR}/packs"
PACK_DIRS=()
while IFS= read -r dir; do
  PACK_DIRS+=("${dir}")
done < <(find "${PACK_ROOT}" -mindepth 1 -maxdepth 1 -type d ! -name components | sort)

if [ "${#PACK_DIRS[@]}" -eq 0 ]; then
  echo "No packs found under ${PACK_ROOT}" >&2
  exit 1
fi

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "${TMP_ROOT}"' EXIT

for dir in "${PACK_DIRS[@]}"; do
  manifest="${dir}/pack.yaml"
  if [ ! -f "${manifest}" ]; then
    echo "Skipping ${dir} (missing pack.yaml)" >&2
    continue
  fi

  name="$(basename "${dir}")"
  gtpack_out="${DIST_DIR}/${name}.gtpack"
  manifest_out="${DIST_DIR}/${name}.cbor"
  sbom_out="${DIST_DIR}/${name}.sbom.json"

  work_dir="${TMP_ROOT}/${name}"
  mkdir -p "${work_dir}"
  rsync -a "${dir}/" "${work_dir}/"
  rm -rf "${work_dir}/components"
  if [ -d "${PACK_ROOT}/components" ]; then
    rsync -a "${PACK_ROOT}/components/" "${work_dir}/components/"
  fi

  echo "Building pack: ${name}"
  "${PACK_BIN}" build \
    --log warn \
    --in "${work_dir}" \
    --gtpack-out "${gtpack_out}" \
    --manifest "${manifest_out}" \
    --sbom "${sbom_out}"
done

echo "Pack artifacts created under ${DIST_DIR}"
