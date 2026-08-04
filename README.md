# DeepRoot

Rust capability microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **1.9.1**

## Learning docs

面向新手的逐步教程（基线 **v1.9.1**）：

- 在线：https://wuyijing-dev.github.io/DeepRoot/
- 本地：`cargo install mdbook && cd book && mdbook serve --open`
- 源码在 `book/`；GitHub Pages 发布目录为 `docs/`（Settings → Pages → **main** / **/docs**）

## Quick start

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/run-qemu.sh          # interactive (shell prompt)
./scripts/run-qemu.sh --smoke  # CI gate
```

Requirements: Rust stable, `qemu-system-riscv64`, `dtc` (device-tree-compiler).

ABI: native capability microkernel (not Linux). Teaching path through **1.9**:
DT + virtio-blk + SMP + richer shell + in-RAM VFS dirs. Stay on **1.9.y** for
FS depth; framebuffer is deferred (see `VERSION`).

Shell: `help`, `ls`, `mkdir`, `cd`, `cat`, `run hello`, pipes `|`, `>`, `&`.

## Layout

```
kernel/           # microkernel (no_std)
libs/             # deeproot-abi, deeproot-user
user/             # init, console, ping, shell, hello…
platform/         # DeepRoot board DTS (qemu-virt/deeproot.dts)
book/             # mdBook source (Chinese learning notes)
docs/             # built site for GitHub Pages (/docs)
scripts/          # QEMU helpers + build-dtb.sh
VERSION           # current release + full roadmap
```

## License

Licensed under [Apache License 2.0](LICENSE).
