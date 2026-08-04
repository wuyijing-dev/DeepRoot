#!/usr/bin/env bash
# run-qemu.sh - boot DeepRoot on QEMU virt + OpenSBI
#
# Usage:
#   ./scripts/run-qemu.sh           # build + run (interactive)
#   ./scripts/run-qemu.sh --gdb     # wait for gdb on :1234
#   ./scripts/run-qemu.sh --smoke   # timed boot; exit 0 if markers appear

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="riscv64gc-unknown-none-elf"
KERNEL_ELF="${ROOT}/target/${TARGET}/release/deeproot-kernel"
MODE="${1:-}"

cd "${ROOT}"
cargo build -p deeproot-kernel --release --target "${TARGET}"

if [[ "${MODE}" == "--smoke" ]]; then
  echo "smoke: running QEMU (30s cap)…"
  LOG="$(mktemp)"
  cleanup() { rm -f "${LOG}"; }
  trap cleanup EXIT
  set +e
  timeout 30 qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -m 128M \
    -nographic \
    -bios default \
    -kernel "${KERNEL_ELF}" \
    >"${LOG}" 2>&1
  set -e
  ok=1
    for needle in \
    "DeepRoot microkernel 1.4.0" \
    "canopy ready" \
    "ping: pong" \
    "hello: spawned ELF says hi" \
    "shell: DeepRoot shell ready" \
    "block: ramdisk ready" \
    "sched: init exited"
  do
    if ! grep -q "${needle}" "${LOG}"; then
      echo "smoke: FAIL missing: ${needle}"
      ok=0
    fi
  done
  if [[ "${ok}" -ne 1 ]]; then
    echo "---- qemu log (tail) ----"
    tail -n 80 "${LOG}"
    exit 1
  fi
  echo "smoke: OK"
  exit 0
fi

GDB_ARGS=()
if [[ "${MODE}" == "--gdb" ]]; then
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
