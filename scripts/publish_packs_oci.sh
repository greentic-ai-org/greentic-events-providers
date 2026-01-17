#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
REGISTRY="${REGISTRY:-ghcr.io}"
OWNER="${OWNER:-greentic-ai}"
REPO="${REPO:-greentic-packs}"
SOURCE_ANNOTATION="https://github.com/greentic-ai/greentic-events-providers"
GITHUB_SHA="${GITHUB_SHA:-$(git -C "${ROOT_DIR}" rev-parse --verify HEAD)}"
MAKE_PUBLIC="${MAKE_PUBLIC:-false}"
GHCR_TOKEN="${GHCR_TOKEN:-${GITHUB_TOKEN:-}}"
VISIBILITY_ENDPOINT="${VISIBILITY_ENDPOINT:-user}"

determine_version() {
  if [ -n "${VERSION:-}" ]; then
    echo "${VERSION}"
    return
  fi

  if tag="$(git -C "${ROOT_DIR}" describe --tags --exact-match 2>/dev/null)"; then
    echo "${tag#v}"
    return
  fi

  version_from_python="$(
    python3 - <<'PY' 2>/dev/null || true
import importlib
import pathlib

try:
    toml = importlib.import_module("tomllib")
except ModuleNotFoundError:
    try:
        toml = importlib.import_module("tomli")
    except ModuleNotFoundError:
        raise SystemExit

root = pathlib.Path(__file__).resolve().parent.parent
data = toml.loads((root / "Cargo.toml").read_text())
print(data.get("workspace", {}).get("package", {}).get("version", ""))
PY
  )"
  if [ -n "${version_from_python}" ]; then
    echo "${version_from_python}"
    return
  fi

  version_from_awk="$(awk '
    $0 ~ /^\[workspace\.package\]/ { in_section=1; next }
    in_section && $0 ~ /^\[/ { in_section=0 }
    in_section && $1 ~ /^version/ {
      gsub(/"/, "", $3);
      print $3;
      exit
    }
  ' "${ROOT_DIR}/Cargo.toml")"
  if [ -n "${version_from_awk}" ]; then
    echo "${version_from_awk}"
    return
  fi
}

VERSION_RESOLVED="$(determine_version)"
if [ -z "${VERSION_RESOLVED}" ]; then
  echo "VERSION not provided and could not determine from git tag or Cargo.toml" >&2
  exit 1
fi

if [ ! -d "${DIST_DIR}" ]; then
  echo "Dist directory ${DIST_DIR} not found. Run scripts/build_packs.sh first." >&2
  exit 1
fi

shopt -s nullglob
PACKS=("${DIST_DIR}"/events-*.gtpack)
if [ "${#PACKS[@]}" -eq 0 ]; then
  echo "No pack artifacts found under ${DIST_DIR}" >&2
  exit 1
fi

for pack in "${PACKS[@]}"; do
  pack_name="$(basename "${pack%.gtpack}")"
  ref="${REGISTRY}/${OWNER}/${REPO}/${pack_name}:${VERSION_RESOLVED}"

  echo "Pushing ${pack_name} -> ${ref}"
  (
    cd "${DIST_DIR}"
    oras push "${ref}" \
    "$(basename "${pack}"):application/vnd.greentic.gtpack+zip" \
    --annotation org.opencontainers.image.source="${SOURCE_ANNOTATION}" \
    --annotation org.opencontainers.image.revision="${GITHUB_SHA}" \
    --annotation org.opencontainers.image.version="${VERSION_RESOLVED}" \
    --annotation org.opencontainers.image.title="${pack_name}"
  )

  digest=""
  if command -v jq >/dev/null 2>&1 && oras manifest fetch --help 2>&1 | grep -q -- "--descriptor"; then
    digest="$(oras manifest fetch --descriptor "${ref}" 2>/dev/null | jq -r '.digest // .Descriptor.digest // empty' || true)"
  fi

  if [ -z "${digest}" ] && command -v sha256sum >/dev/null 2>&1; then
    digest="$(oras manifest fetch "${ref}" 2>/dev/null | sha256sum | awk '{print "sha256:"$1}' || true)"
  fi

  if [ -n "${digest}" ]; then
    echo "Digest for ${ref}: ${digest}"
  else
    echo "Digest for ${ref}: (unavailable - oras digest lookup failed)" >&2
  fi

  if [ "${MAKE_PUBLIC}" = "true" ] && [ -n "${GHCR_TOKEN}" ]; then
    encoded_package="${REPO}%2F${pack_name}"
    echo "Setting visibility public for ${OWNER}/${encoded_package}"
    whoami_tmp="$(mktemp)"
    whoami_code="$(
      curl -sS -o "${whoami_tmp}" -w "%{http_code}" \
        -H "Authorization: Bearer ${GHCR_TOKEN}" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/user" || true
    )"
    if [ "${whoami_code}" = "200" ]; then
      whoami_login="$(python3 - <<'PY'
import json
import sys

path = sys.argv[1]
data = json.load(open(path, "r", encoding="utf-8"))
login = data.get("login", "")
uid = data.get("id", "")
print(f"Token identity: {login} (id={uid})")
PY
      "${whoami_tmp}" 2>/dev/null || true)"
      if [ -n "${whoami_login}" ]; then
        echo "${whoami_login}"
      fi
    else
      echo "Token identity check failed with status ${whoami_code}" >&2
    fi
    rm -f "${whoami_tmp}"
    if [ "${VISIBILITY_ENDPOINT}" = "user" ]; then
      visibility_url="https://api.github.com/user/packages/container/${encoded_package}/visibility"
    else
      visibility_url="https://api.github.com/${VISIBILITY_ENDPOINT}/${OWNER}/packages/container/${encoded_package}/visibility"
    fi
    echo "Visibility URL: ${visibility_url}"
    response_tmp="$(mktemp)"
    http_code="$(
      curl -sS -o "${response_tmp}" -w "%{http_code}" -X PATCH \
        -H "Authorization: Bearer ${GHCR_TOKEN}" \
        -H "Accept: application/vnd.github+json" \
        "${visibility_url}" \
        -d '{"visibility":"public"}' || true
    )"
    if [ "${http_code}" != "200" ]; then
      echo "Visibility update failed with status ${http_code} for ${visibility_url}" >&2
      cat "${response_tmp}" >&2
      rm -f "${response_tmp}"
      exit 1
    fi
    rm -f "${response_tmp}"
  fi
done
