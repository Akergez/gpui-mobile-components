#!/usr/bin/env bash
# Runs cargo where the toolkit's native libraries are: on this machine the
# host has no fontconfig/xkbcommon headers, the `backseating-dev` distrobox
# (which shares $HOME) does. Elsewhere, plain cargo.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if pkg-config --exists fontconfig 2>/dev/null || [[ -n ${CONTAINER_ID:-} ]]; then
  cd "$root" && exec cargo "$@"
fi
exec distrobox enter backseating-dev -- bash -c 'cd "$1" && shift && cargo "$@"' _ "$root" "$@"
