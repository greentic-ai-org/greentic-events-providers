#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v greentic-provision >/dev/null 2>&1; then
  echo "greentic-provision not found. Install with: cargo binstall greentic-provision --locked" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for provisioning conformance checks." >&2
  exit 1
fi

PACKS=(
  "events-email"
  "events-sms"
  "events-webhook"
  "events-timer"
  "events-dummy"
)

declare -A PACK_IDS=(
  ["events-email"]="greentic.events.email"
  ["events-sms"]="greentic.events.sms"
  ["events-webhook"]="greentic.events.webhook"
  ["events-timer"]="greentic.events.timer"
  ["events-dummy"]="greentic.events.provider.dummy"
)

declare -A REQUIRES_BASE_URL=(
  ["events-sms"]="true"
  ["events-webhook"]="true"
)

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

for pack in "${PACKS[@]}"; do
  pack_dir="${ROOT_DIR}/packs/${pack}"
  fixtures_dir="${pack_dir}/fixtures"
  answers="${fixtures_dir}/setup.input.json"
  expected="${fixtures_dir}/setup.expected.plan.json"
  requirements_expected="${fixtures_dir}/requirements.expected.json"

  if [ ! -f "${pack_dir}/pack.json" ]; then
    echo "Missing pack.json for ${pack_dir}" >&2
    exit 1
  fi
  if [ ! -f "${requirements_expected}" ]; then
    echo "Missing requirements.expected.json for ${pack}" >&2
    exit 1
  fi
  has_setup="$(PACK_DIR="${pack_dir}" python3 - <<'PY'
import json
import os
from pathlib import Path

pack_json = Path(os.environ["PACK_DIR"]) / "pack.json"
data = json.loads(pack_json.read_text())
entry_flows = data.get("meta", {}).get("entry_flows", {})
setup = None
if isinstance(entry_flows, dict):
    setup = entry_flows.get("setup")
elif isinstance(entry_flows, list):
    for flow in entry_flows:
        entry = flow.get("entry") or flow.get("name")
        if entry == "setup":
            setup = flow.get("id") or flow.get("flow_id") or flow.get("name")
            break
print("yes" if setup else "no")
PY
)"
  if [ "${has_setup}" != "yes" ]; then
    has_no_setup="$(PACK_DIR="${pack_dir}" python3 - <<'PY'
import json
import os
from pathlib import Path

pack_json = Path(os.environ["PACK_DIR"]) / "pack.json"
data = json.loads(pack_json.read_text())
caps = data.get("meta", {}).get("capabilities", [])
print("yes" if "provisioning:none" in caps else "no")
PY
)"
    if [ "${has_no_setup}" != "yes" ]; then
      echo "Pack ${pack} has no setup entry but is missing provisioning:none capability." >&2
      exit 1
    fi
    continue
  fi

  requirements_actual="${TMP_DIR}/${pack}.requirements.json"
  PACK_DIR="${pack_dir}" python3 - <<'PY' > "${requirements_actual}"
import codecs
import json
import os
import re
from pathlib import Path

wat_path = Path(os.environ["PACK_DIR"]) / "setup_default__requirements.wat"
text = wat_path.read_text()
match = re.search(r'\(data \(i32\.const 0\) "(.*?)"\)', text, re.DOTALL)
if not match:
    raise SystemExit("missing requirements data segment")
payload_escaped = match.group(1)
payload = codecs.decode(payload_escaped, "unicode_escape")
parsed = json.loads(payload)
print(json.dumps(parsed))
PY
  jq -S '.' "${requirements_expected}" > "${TMP_DIR}/${pack}.requirements.expected.json"
  jq -S '.' "${requirements_actual}" > "${TMP_DIR}/${pack}.requirements.actual.json"
  if ! diff -u "${TMP_DIR}/${pack}.requirements.expected.json" "${TMP_DIR}/${pack}.requirements.actual.json"; then
    echo "Requirements output mismatch for ${pack}" >&2
    exit 1
  fi
  if [ ! -f "${answers}" ]; then
    echo "Missing setup.input.json for ${pack}" >&2
    exit 1
  fi
  if [ ! -f "${expected}" ]; then
    echo "Missing setup.expected.plan.json for ${pack}" >&2
    exit 1
  fi

  base_url_args=()
  if [ "${REQUIRES_BASE_URL[$pack]-false}" = "true" ]; then
    base_url_args=(--public-base-url "https://example.invalid")
  fi

  output_path="${TMP_DIR}/${pack}.json"
  greentic-provision dry-run setup \
    --pack "${pack_dir}" \
    --provider-id "${PACK_IDS[$pack]}" \
    --install-id "${PACK_IDS[$pack]}-fixture" \
    "${base_url_args[@]}" \
    --answers "${answers}" \
    --json > "${output_path}"

  actual_path="${TMP_DIR}/${pack}.plan.json"
  expected_path="${TMP_DIR}/${pack}.expected.json"
  subscriptions_expected="${fixtures_dir}/subscriptions.expected.json"

  jq -S '.plan' "${output_path}" > "${actual_path}"
  jq -S '.' "${expected}" > "${expected_path}"

  if ! diff -u "${expected_path}" "${actual_path}"; then
    echo "Provisioning plan mismatch for ${pack}" >&2
    exit 1
  fi
  if [ -f "${subscriptions_expected}" ]; then
    jq -S '.plan.subscription_ops' "${output_path}" > "${TMP_DIR}/${pack}.subscriptions.actual.json"
    jq -S '.' "${subscriptions_expected}" > "${TMP_DIR}/${pack}.subscriptions.expected.json"
    if ! diff -u "${TMP_DIR}/${pack}.subscriptions.expected.json" "${TMP_DIR}/${pack}.subscriptions.actual.json"; then
      echo "Subscriptions mismatch for ${pack}" >&2
      exit 1
    fi
  fi

done

echo "Provisioning conformance checks passed."
