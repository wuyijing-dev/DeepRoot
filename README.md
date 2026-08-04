# DeepRoot

Rust capability microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **1.4.0**

## Quick start

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/run-qemu.sh          # interactive (shell prompt)
./scripts/run-qemu.sh --smoke  # CI gate
```

Requirements: Rust stable, `qemu-system-riscv64`.

ABI: native capability microkernel (not Linux). Teaching path through **1.4**:
per-task AS + `SYS_SPAWN` / `SYS_EXEC`, serial shell, ramfs (text + ELF), ramdisk stand-in.

Shell: `ls`, `cat readme.txt`, `run hello`, `run badapple` (realtime ASCII).

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
