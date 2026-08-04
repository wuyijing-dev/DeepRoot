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
