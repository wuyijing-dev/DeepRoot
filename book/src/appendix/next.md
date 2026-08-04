# 下一步可以看什么

你已经走完 DeepRoot **1.4** 教学主线（当前推荐标签 **`v1.4.1`**）。下面按「投入产出比」排序，任选。

## 1. 巩固（强烈建议）

- 不看文档，独立画出：`ecall` → `trap` → `handle_syscall` → `sret`  
- 按 [自己写用户程序](../hands-on/write-user-prog.md) 做一个新 ELF  
- 对照 `ls`：分清哪些文件来自 embed、哪些来自 DRFS  
- 把 [名词表](glossary.md) 里仍模糊的词对照源码再读一遍  

## 2. 在仓库里继续挖（仍属 DeepRoot）

| 方向 | 从哪开始 |
|---|---|
| 真·virtio-blk | 换掉 `DISK[]` 后端；保留 DRFS / `fs` 路径 API |
| 块上 ELF | 让 `SYS_EXEC` 也能从 DRFS 读 ELF（今天只 embed） |
| 更丰富的 IPC | `ipc.rs`、cap grant、多客户共享服务 |
| 多核 | `VERSION` 里 multi-hart 相关条目；今天仍以单 hart 为主 |

改之前先读 `VERSION`：哪些是已承诺范围，哪些是你自己的实验分支。

## 3. 对照其它教材（换口味）

- **xv6**（RISC-V）：经典教学宏内核，对比「系统调用长什么样」  
- **rCore / uCore**：同为 Rust + RISC-V 的课程向 OS  
- **seL4 / Fiasco.OC 文档**：看工业能力微内核如何谈证明与隔离（难度陡增）  

对照时问三个问题：

1. 谁拥有页表？  
2. 谁拥有设备驱动？  
3. 权限是 UID 还是 capability？

## 4. 不建议的下一步

- 一上来移植 bash / glibc  
- 为了「像 Linux」强行加一堆 POSIX 皮  
- 把 Bad Apple 当内核正确性证明  

## 5. 参与项目

远程：`git@github.com:wuyijing-dev/DeepRoot.git`。发 issue / PR 前先说明你对齐的标签（`v1.4.1` 等）。
