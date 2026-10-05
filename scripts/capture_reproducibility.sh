#!/usr/bin/env bash
set -euo pipefail

source_revision="${1:?source revision required}"
out="${2:-results/ci/reproducibility.txt}"
mkdir -p "$(dirname "$out")"

lock_sha256="$(sha256sum Cargo.lock | awk '{print $1}')"
{
  printf 'schema=fieldlab.reproducibility.v1\n'
  printf 'source_revision=%s\n' "$source_revision"
  printf 'cargo_lock_sha256=%s\n' "$lock_sha256"
  printf 'fieldlab_toolchain=nightly-2026-07-02\n'
  rustc +nightly-2026-07-02 --version --verbose | sed 's/^/rustc./'
  cargo +nightly-2026-07-02 --version | sed 's/^/cargo./'
} > "$out"

cat "$out"
