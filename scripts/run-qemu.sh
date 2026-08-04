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

# Host is usually x86_64: RISC-V guests run under TCG. Prefer multi-thread TCG
# and a large TB cache so ASCII playback is less jerky.
QEMU_ACCEL=(-accel tcg,thread=multi)
QEMU_COMMON=(
  -machine virt
  -cpu rv64
  -smp 1
  -m 256M
  -nographic
  -bios default
  -kernel "${KERNEL_ELF}"
)

if [[ "${MODE}" == "--smoke" ]]; then
  echo "smoke: running QEMU (30s cap)…"
  LOG="$(mktemp)"
  cleanup() { rm -f "${LOG}"; }
  trap cleanup EXIT
  set +e
  timeout 30 qemu-system-riscv64 \
    "${QEMU_ACCEL[@]}" \
    "${QEMU_COMMON[@]}" \
    >"${LOG}" 2>&1
  set -e
  ok=1
    for needle in \
    "DeepRoot microkernel 1.4.1" \
    "canopy ready" \
    "ping: pong" \
    "hello: spawned ELF says hi" \
    "shell: DeepRoot shell ready" \
    "block: ramdisk ready" \
    "DRFS" \
    "init: handing off to shell"
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
  "${QEMU_ACCEL[@]}" \
  "${QEMU_COMMON[@]}" \
  "${GDB_ARGS[@]}"
