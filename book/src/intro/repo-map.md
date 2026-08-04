# 仓库长什么样（详细地图）

第一次打开仓库时，先建立「地图」，**不要**从头到尾通读所有 `.rs`。

## 1. 顶层一览

```text
DeepRoot/
├── VERSION                 # 第一行=当前版本；下文=完整路线图
├── README.md               # 给 GitHub 访客的简介
├── LICENSE                 # Apache-2.0
├── Cargo.toml              # workspace：列出 kernel / libs / user/*
├── platform/               # 板级设备树源（qemu-virt/deeproot.dts）
├── book/                   # 学习文档源码（mdBook）
├── docs/                   # 构建后的静态站点（GitHub Pages → /docs）
├── scripts/
│   ├── run-qemu.sh         # 构建 DTB + 内核 + 启动 QEMU
│   └── build-dtb.sh        # dtc：DTS → build/*.dtb
├── kernel/                 # 微内核 crate（#![no_std]）
│   ├── build.rs            # 编译并嵌入用户 ELF
│   ├── linker.ld
│   └── src/
│       ├── main.rs         # kernel_main
│       ├── boot.rs         # _start
│       ├── fdt.rs          # FDT 遍历 → Platform
│       ├── virtio_blk.rs   # legacy virtio-mmio 块驱动
│       ├── sbi.rs          # 控制台 / 时钟等 SBI
│       ├── trap.rs         # 陷阱与 syscall 入口
│       ├── sched.rs        # 调度 + 大部分 syscall 实现
│       ├── mm/             # 内存与页表
│       ├── cap/            # 能力
│       ├── ipc.rs / ledger.rs / elf.rs / fs.rs / block.rs …
│       └── servers.rs      # 装载用户服务器并 enter_first
├── libs/
│   ├── deeproot-abi/       # syscall 号、错误码、IPC 结构（用户+内核共用）
│   └── deeproot-user/      # 用户态 ecall 封装
└── user/
    ├── init/               # 根服务器：演示 IPC，再把舞台留给 shell
    ├── console/            # 控制台服务
    ├── ping/               # IPC ping/pong
    ├── hello/              # 可 spawn / run 的最小程序
    ├── shell/              # 交互 shell
    └── badapple/           # 可选彩蛋（非主线）
```

## 2. 「我想搞懂 X → 打开这些文件」

| 你想搞懂… | 先打开 |
|---|---|
| 开机入口 | `kernel/src/boot.rs`、`main.rs`、`sbi.rs` |
| 页表 / 内存 | `kernel/src/mm/`（`memmap` → `frame` → `sv39`） |
| 设备树 | `platform/qemu-virt/deeproot.dts`、`kernel/src/fdt.rs` |
| 能力 | `kernel/src/cap/`、`libs/deeproot-abi` |
| IPC / Ledger | `kernel/src/ipc.rs`、`ledger.rs`、`user/init`、`user/ping` |
| 调度与 syscall | `kernel/src/sched.rs`、`trap.rs` |
| 用户程序怎么进内核 | `libs/deeproot-user/src/lib.rs` |
| shell / ramfs | `user/shell/`、`kernel/src/fs.rs`、`kernel/build.rs` |
| 块 / virtio | `kernel/src/block.rs`、`virtio_blk.rs` |

## 3. 构建时发生了什么？（必读）

```text
./scripts/run-qemu.sh
    → scripts/build-dtb.sh（dtc：deeproot.dts → .dtb）
    → cargo build -p deeproot-kernel …
        → kernel/build.rs 对每个 user 包再 cargo build
        → 把 ELF 复制到 OUT_DIR
        → fs.rs / servers.rs 里 include_bytes!
    → qemu … -kernel … -dtb … -device virtio-blk-device …
```

因此：

- **改用户程序 ≈ 要重编内核**（嵌入的字节才会变）  
- ramfs 里的 `hello` **不是**你拷进虚拟磁盘的文件  

细节在 [1.3 ramfs](../path/08-fs.md) 与 [写用户程序](../hands-on/write-user-prog.md)。

## 4. 文档目录怎么对应源码？

| 文档章节 | 主要源码 |
|---|---|
| 0.1 | `boot` / `sbi` / `console` / 早期 `trap` |
| 0.2 | `mm/*` |
| 0.3–0.4 | `cap/*`、`ipc`、`ledger` |
| 0.5–0.6 | `servers`、`sched`、`timer`、`user/*` |
| 1.0 | `deeproot-abi` |
| 1.1 | `elf`、`AddrSpace`、`SYS_SPAWN` |
| 1.2 | `user/shell`、`SYS_DEBUG_READ` |
| 1.3 | `fs.rs`、`SYS_EXEC` |
| 1.4 | `block.rs`（DRFS） |
| 1.5–1.6 | `platform/…/deeproot.dts`、`fdt.rs`、`virtio_blk.rs` |

下一章回主线：[学习路线图](../path/overview.md)。若尚未开机，先去 [第一次启动](first-boot.md)。
