#!/usr/bin/env bash
set -euo pipefail

OFFLINE=1
RUN_SECURITY=1

usage() {
  cat <<'EOF'
Safer local test runner for solunatus.

Usage:
  scripts/safe_local_test.sh [options]

Options:
  --allow-network   Disable offline mode for cargo.
  --skip-security   Skip optional `cargo audit` pre-check.
  -h, --help        Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-network)
      OFFLINE=0
      ;;
    --skip-security)
      RUN_SECURITY=0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [[ ! -f Cargo.toml || ! -f Cargo.lock ]]; then
  echo "Run this script from the repository root." >&2
  exit 1
fi

echo "==> Preflight: dependency diffs"
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  dep_changes="$(git diff --name-only HEAD -- Cargo.toml Cargo.lock)"
  if [[ -n "${dep_changes}" ]]; then
    echo "Dependency files changed in working tree:" >&2
    echo "${dep_changes}" >&2
    echo "Review dependency diffs before trusting local execution." >&2
  else
    echo "No local dependency file changes detected."
  fi
fi

echo "==> Preflight: scrub common credential env vars"
unset GITHUB_TOKEN GH_TOKEN CARGO_REGISTRIES_CRATES_IO_TOKEN
unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN
unset OPENAI_API_KEY ANTHROPIC_API_KEY

default_cargo_home="${CARGO_HOME:-${HOME}/.cargo}"
if mkdir -p "${default_cargo_home}" 2>/dev/null; then
  export CARGO_HOME="${default_cargo_home}"
else
  export CARGO_HOME="${PWD}/.cargo-local"
  mkdir -p "${CARGO_HOME}"
fi

export CARGO_TARGET_DIR="${PWD}/target-safe-local"

test_flags=(--locked)
if [[ ${OFFLINE} -eq 1 ]]; then
  export CARGO_NET_OFFLINE=true
  test_flags+=(--offline)
  echo "Offline mode enabled (use --allow-network to disable)."
fi

if [[ ${RUN_SECURITY} -eq 1 ]]; then
  if command -v cargo-audit >/dev/null 2>&1; then
    echo "==> Security check: cargo audit"
    if ! cargo audit; then
      if [[ ${OFFLINE} -eq 1 ]]; then
        echo "Warning: cargo audit failed in offline mode. Re-run with --allow-network to refresh advisories." >&2
      else
        exit 1
      fi
    fi
  else
    echo "Skipping security check (cargo-audit not installed)."
  fi
fi

echo "==> Running test suite"
cargo test "${test_flags[@]}" --verbose

echo "==> Running doc tests"
cargo test "${test_flags[@]}" --doc --verbose

echo "==> Safe local test run complete"
