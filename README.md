# DeepRoot

Rust capability microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **1.0.0**

## Quick start

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/run-qemu.sh          # interactive
./scripts/run-qemu.sh --smoke  # CI gate: canopy + pong + init exit
```

Requirements: Rust stable, `qemu-system-riscv64`.

ABI numbers live in `libs/deeproot-abi` and are frozen for 1.0 (additive only in 1.1+).
DeepRoot stays on a **native capability ABI** (not Linux-compatible).

Roadmap after 1.0 (slow series — detail in [`VERSION`](VERSION)):
**1.1** process/ELF spawn → **1.2** serial console + native shell → **1.3** FS server + path run.
Work items under each series are tracked there; VERSION only bumps when a series is usable.

## Layout

```
kernel/           # microkernel (no_std)
libs/             # deeproot-abi, deeproot-user
user/             # init, console, ping (U-mode ELFs)
scripts/          # QEMU helpers
VERSION           # current release + full roadmap
```

## License

Licensed under [Apache License 2.0](LICENSE).
