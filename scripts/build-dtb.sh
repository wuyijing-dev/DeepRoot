#!/usr/bin/env bash
# build-dtb.sh — compile DeepRoot's in-tree DTS to a DTB blob
#
# Source of truth: platform/qemu-virt/deeproot.dts
# Output:          build/deeproot-qemu-virt.dtb
#
# Requires: device-tree-compiler (`dtc`).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DTS="${ROOT}/platform/qemu-virt/deeproot.dts"
OUT_DIR="${ROOT}/build"
DTB="${OUT_DIR}/deeproot-qemu-virt.dtb"

if ! command -v dtc >/dev/null 2>&1; then
  echo "build-dtb: missing dtc (install device-tree-compiler)" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"
dtc -I dts -O dtb -o "${DTB}" "${DTS}"
echo "build-dtb: ${DTB} ($(wc -c <"${DTB}") bytes)"
