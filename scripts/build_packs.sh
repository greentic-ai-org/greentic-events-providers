#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist/packs"

PACKC_BIN="$(command -v packc || true)"
if [ -z "${PACKC_BIN}" ]; then
  echo "packc not found. Install with: cargo install greentic-pack --locked --bin packc" >&2
  exit 1
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

  # copy manifest as pack.yaml and supporting flows
  cp "${pack}" "${work_dir}/pack.yaml"
  rsync -a "${ROOT_DIR}/flows" "${work_dir}/"

  echo "Building pack: ${pack} -> ${out}"
  "${PACKC_BIN}" build \
    --in "${work_dir}" \
    --gtpack-out "${out}" \
    --manifest "${DIST_DIR}/${name}.cbor" \
    --sbom "${DIST_DIR}/${name}.sbom.json"
done

echo "Pack artifacts created under ${DIST_DIR}"
