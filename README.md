# DeepRoot

Rust capability microkernel for RISC-V.

Repository: https://github.com/wuyijing-dev/DeepRoot  
Version: see [`VERSION`](VERSION) — current **1.15.2**

## Learning docs

面向新手的逐步教程（基线 **v1.15.2**）：

- 在线：https://wuyijing-dev.github.io/DeepRoot/
- 本地：`cargo install mdbook && cd book && mdbook serve --open`
- 源码在 `book/`；GitHub Pages 发布目录为 `docs/`（Settings → Pages → **main** / **/docs**）

## Quick start

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/run-qemu.sh          # interactive (serial, -nographic)
./scripts/run-qemu.sh --gui    # GTK window + serial (ramfb / fbmenu)
./scripts/run-qemu.sh --smoke  # CI gate
```

Requirements: Rust stable, `qemu-system-riscv64`, `dtc` (device-tree-compiler).

ABI: native capability microkernel (not Linux). Teaching path through **1.15**:
framebuffer (`/fbdemo`, `/fbmenu` under `drivers/`). See `VERSION` for roadmap.

Shell: `help`, `ls`, `mkdir`, `cd`, `modload`, `run hello`, `|`, `>`, `&`.

## Layout

```
kernel/           # microkernel (no_std)
libs/             # deeproot-abi, deeproot-user
drivers/          # userspace device servers (virtioblk, fbdemo, fbmenu, …)
user/             # init, console, ping, shell, hello, demos…
platform/         # DeepRoot board DTS (qemu-virt/deeproot.dts)
book/             # mdBook source (Chinese learning notes)
docs/             # built site for GitHub Pages (/docs)
scripts/          # QEMU helpers + build-dtb.sh
VERSION           # current release + full roadmap
```

## License

Licensed under [Apache License 2.0](LICENSE).
