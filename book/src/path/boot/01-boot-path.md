# 0.1.1 从 QEMU 到 `_start`

这一页只讲：**按下 `./scripts/run-qemu.sh` 以后，控制权是怎样进入 DeepRoot 的。**

## 1. 时间线

```text
主机 shell
  -> run-qemu.sh
  -> qemu-system-riscv64
  -> OpenSBI (M-mode)
  -> DeepRoot `_start` (S-mode)
```

## 2. 关键文件

- `scripts/run-qemu.sh`
- `kernel/src/boot.rs`

## 3. 你要看懂的点

- `-machine virt`：QEMU 的虚拟开发板
- `-bios default`：让 OpenSBI 先起来
- `-kernel ...`：把内核 ELF 交给 QEMU
- `_start`：真正进入你自己的内核入口

## 4. 为什么新手要先会这一页

因为如果系统连 `_start` 都没进：

- 后面的 trap、页表、shell 文档都没意义
- 你要查的不是 Rust 逻辑，而是启动链接与 QEMU/OpenSBI 约定

下一页：[0.1.2 跟读 `boot.rs`](02-boot-rs.md)

