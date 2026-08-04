# 下一步可以看什么

你已经走完 DeepRoot **1.6** 教学主线（当前推荐标签 **`v1.6.1`**）：自有 DTS、FDT 发现、virtio-blk 上的 DRFS。  
仓库路线图继续到 **2.0**（见根目录 `VERSION`）：SMP → 更完善自研 shell → framebuffer 简易 UI。

## 1. 巩固（强烈建议）

- 不看文档，独立画出：`ecall` → `trap` → `handle_syscall` → `sret`  
- 按 [自己写用户程序](../hands-on/write-user-prog.md) 做一个新 ELF  
- 对照 `ls`：分清哪些文件来自 embed、哪些来自 DRFS  
- 把 [名词表](glossary.md) 里仍模糊的词对照源码再读一遍  

## 2. 官方下一站（1.5 → 2.0）

按 `VERSION` 顺序推进（实现前请先读该文件里的 W1…验收条）：

| 系列 | 用户可见目标 |
|---|---|
| **1.5** | FDT 发现 UART / virtio / memory（少硬编码） |
| **1.6** | 真 virtio-blk（或等价）接在现有 FS/DRFS 路径后 |
| **1.7** | QEMU `-smp 2` 多 hart 调度 |
| **1.8** | **自研**更完善 shell（argv/环境/history/`&`/简单管道）；**不**移植 bash |
| **1.9** | Framebuffer：清屏、画点/矩形、简单菜单或图形终端（不做桌面） |
| **2.0.0** | 以上能力集成发布与文档基线 |

本地实验也可从下表挑一块先挖（仍建议对齐系列边界再 bump `VERSION`）：

| 方向 | 从哪开始 |
|---|---|
| 设备树 | `kernel/src/mm/memmap.rs` 的 FDT 雏形 → 计划中的 `fdt` 模块 |
| 真·virtio-blk | `kernel/src/block.rs`；保留路径 API |
| 多核 | `timer` / `sched` / `ipc` 里的单 hart 假设 |
| 图形 | QEMU ramfb / virtio-gpu（等 1.5/1.7 更稳再上更轻松） |

## 3. 对照其它教材（换口味）

- **xv6**（RISC-V）：经典教学宏内核，对比「系统调用长什么样」  
- **rCore / uCore**：同为 Rust + RISC-V 的课程向 OS  
- **seL4 / Fiasco.OC 文档**：看工业能力微内核如何谈证明与隔离（难度陡增）  

对照时问三个问题：

1. 谁拥有页表？  
2. 谁拥有设备驱动？  
3. 权限是 UID 还是 capability？

## 4. 不建议的下一步

- 一上来移植 bash / glibc / 完整 POSIX sh  
- 为了「像 Linux」强行加一堆 POSIX 皮  
- 把 Bad Apple 或完整桌面 WM 当 2.0 必达项  

## 5. 参与项目

远程：`git@github.com:wuyijing-dev/DeepRoot.git`。发 issue / PR 前先说明你对齐的标签（`v1.4.1` 等）或目标系列（如 1.5）。
