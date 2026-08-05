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
DISK_IMG="${ROOT}/build/deeproot-disk.img"
DTB="${ROOT}/build/deeproot-qemu-virt.dtb"
MODE="${1:-}"

cd "${ROOT}"

# DeepRoot's own device tree (not QEMU auto-generated).
chmod +x "${ROOT}/scripts/build-dtb.sh"
"${ROOT}/scripts/build-dtb.sh"

cargo build -p deeproot-kernel --release --target "${TARGET}"

mkdir -p "${ROOT}/build"
if [[ ! -f "${DISK_IMG}" ]]; then
  # 1 MiB raw image; kernel formats DRFS into the first 64 KiB if empty.
  dd if=/dev/zero of="${DISK_IMG}" bs=1M count=1 status=none
  echo "run-qemu: created ${DISK_IMG}"
fi

# Host is usually x86_64: RISC-V guests run under TCG. Prefer multi-thread TCG
# and a large TB cache so ASCII playback is less jerky.
QEMU_ACCEL=(-accel tcg,thread=multi)
QEMU_COMMON=(
  -machine virt
  -cpu rv64
  -smp 2
  -m 256M
  -nographic
  -bios default
  -kernel "${KERNEL_ELF}"
  -dtb "${DTB}"
  -drive "file=${DISK_IMG},if=none,format=raw,id=hd0"
  -device virtio-blk-device,drive=hd0,bus=virtio-mmio-bus.0
)

if [[ "${MODE}" == "--smoke" ]]; then
  # Fresh image so boot1 formats DRFS; boot2 proves durable.txt survived.
  dd if=/dev/zero of="${DISK_IMG}" bs=1M count=1 status=none
  echo "smoke: boot1 (format + write durable.txt, 30s)…"
  LOG1="$(mktemp)"
  LOG2="$(mktemp)"
  cleanup() { rm -f "${LOG1}" "${LOG2}"; }
  trap cleanup EXIT

  check_needles() {
    local log="$1"
    shift
    local ok=1 needle
    for needle in "$@"; do
      if ! grep -q "${needle}" "${log}"; then
        echo "smoke: FAIL missing: ${needle}"
        ok=0
      fi
    done
    [[ "${ok}" -eq 1 ]]
  }

  set +e
  timeout 30 qemu-system-riscv64 \
    "${QEMU_ACCEL[@]}" \
    "${QEMU_COMMON[@]}" \
    >"${LOG1}" 2>&1
  set -e

  COMMON_NEEDLES=(
    "DeepRoot microkernel 1.14.2"
    "fdt: model \"DeepRoot QEMU virt\""
    "fdt: board deeproot,qemu-virt"
    "fdt: cpu count=2"
    "smp: 2 hart(s) online"
    "smp: secondary hart="
    "fdt: virtio-mmio count="
    "virtio-blk: ready"
    "block: virtio-blk ready"
    "vfs: in-RAM tree ready"
    "DRFS"
    "canopy ready"
    "ping: pong"
    "hello: spawned ELF says hi"
    "module: loaded 'moddemo'"
    "moddemo: online"
    "moddemo: pong"
    "init: module loaded"
    "init: module call ok"
    "init: cp modnote -> mynote ok"
    "module: loaded 'mynote'"
    "modnote: online"
    "modnote: noted"
    "init: vfs module loaded"
    "init: vfs module call ok"
    "init: lookup ping ok"
    "service: resolved 'mynote'"
    "init: lookup mynote ok"
    "module: loaded 'ping'"
    "init: durable DRFS written"
    "init: fd read ok"
    "init: slept"
    "init: time_ms ok"
    "Root Ledger"
    "init: ledger dumped"
    "CapSpace"
    "init: caps dumped"
    "grant: alloc frame"
    "grant: mapped into sched="
    "grantpeer: online"
    "grantpeer: saw magic"
    "init: grant peer ok"
    "grant: unmapped sched="
    "grant: revoked frame"
    "init: frame revoke ok"
    "grant: mmio frame pa="
    "virtioblk: probe start"
    "virtioblk: found block device"
    "virtioblk: probe ok"
    "init: virtioblk loaded"
    "shell: DeepRoot shell 1.14 ready"
    "init: handing off to shell"
  )

  ok=1
  if ! check_needles "${LOG1}" "${COMMON_NEEDLES[@]}" \
      "block: formatted DRFS image"; then
    ok=0
  fi
  if [[ "${ok}" -ne 1 ]]; then
    echo "---- qemu boot1 log (tail) ----"
    tail -n 160 "${LOG1}"
    exit 1
  fi

  echo "smoke: boot2 (existing DRFS + survived reboot, 30s)…"
  set +e
  timeout 30 qemu-system-riscv64 \
    "${QEMU_ACCEL[@]}" \
    "${QEMU_COMMON[@]}" \
    >"${LOG2}" 2>&1
  set -e

  if ! check_needles "${LOG2}" "${COMMON_NEEDLES[@]}" \
      "block: found existing DRFS image" \
      "block: durable.txt survived reboot"; then
    echo "---- qemu boot2 log (tail) ----"
    tail -n 160 "${LOG2}"
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
