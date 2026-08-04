# DeepRoot

Rust capability microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **1.4.0**

## Learning docs

面向新手的逐步教程（基线 **v1.4.0**）：

- 在线：https://wuyijing-dev.github.io/DeepRoot/
- 本地：`cargo install mdbook && cd docs && mdbook serve --open`

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
user/             # init, console, ping, shell, hello…
docs/             # mdBook learning notes (Chinese)
scripts/          # QEMU helpers
VERSION           # current release + full roadmap
```

## License

Licensed under [Apache License 2.0](LICENSE).
