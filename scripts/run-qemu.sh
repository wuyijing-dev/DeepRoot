#!/usr/bin/env bash
# run-qemu.sh - boot DeepRoot on QEMU virt + OpenSBI
#
# Usage:
#   ./scripts/run-qemu.sh           # build + run
#   ./scripts/run-qemu.sh --gdb     # wait for gdb on :1234

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="riscv64gc-unknown-none-elf"
KERNEL_ELF="${ROOT}/target/${TARGET}/release/deeproot-kernel"

cd "${ROOT}"
cargo build -p deeproot-kernel --release --target "${TARGET}"

GDB_ARGS=()
if [[ "${1:-}" == "--gdb" ]]; then
  GDB_ARGS=(-S -s)
  echo "QEMU waiting for GDB on :1234"
fi

exec qemu-system-riscv64 \
  -machine virt \
  -cpu rv64 \
  -m 128M \
  -nographic \
  -bios default \
  -kernel "${KERNEL_ELF}" \
  "${GDB_ARGS[@]}"
