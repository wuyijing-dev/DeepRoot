# DeepRoot

Educational Rust microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **0.5.5** Server Grove  

## Quick start

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/run-qemu.sh
```

Requirements: Rust stable, `qemu-system-riscv64`.

## Layout

```
kernel/           # microkernel (no_std)
libs/             # deeproot-abi, deeproot-user
user/             # init, console, ping (U-mode ELFs)
scripts/          # QEMU helpers
.cursor/rules/    # project policy
VERSION           # current release + full roadmap
```

## License

Licensed under [Apache License 2.0](LICENSE).
