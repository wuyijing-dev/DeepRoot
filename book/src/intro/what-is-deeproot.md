# 这是什么？

DeepRoot 是一个用 **Rust** 写的、跑在 **RISC-V** 上的**能力微内核**。  
这份说明写给完全新手：你会慢慢碰到「特权级」「页表」「系统调用」这些词——先建立图像，细节后面按版本拆开讲。

## 1. 用一家咖啡馆类比操作系统

想象操作系统是一家店：

| 角色 | 宏内核（如 Linux）大致像 | DeepRoot 微内核大致像 |
|---|---|---|
| 老板（内核） | 自己做咖啡、收银、扫地、修咖啡机 | 只定规矩：谁能进后厨、怎么传话、怎么排班 |
| 员工（用户态） | 较少；很多事老板亲力亲为 | barista / 收银员是**独立程序**：console、ping、shell… |
| 权限 | 「你是不是店长/员工编号多少」 | 「你手里有没有这张**工作证（capability）**」 |
| 说话方式 | 很多标准窗口（POSIX） | 店内对讲机协议（**自己的 syscall / IPC**） |

所以：学 DeepRoot 时，不要先问「`open()` 的 Linux 手册哪一页」，而要问：

1. 这件事是内核做还是用户态服务器做？  
2. 调用方有没有能力票？  
3. 消息怎么通过 IPC 或薄薄的 syscall 传过去？

## 2. 三种特权级（RISC-V 最小必会）

CPU 不会让所有代码权限相同。DeepRoot 这条启动链上你至少要知道：

```text
M-mode（Machine）  ← OpenSBI 固件在这里
    │
    ▼
S-mode（Supervisor）← DeepRoot 内核在这里
    │
    ▼
U-mode（User）      ← init / shell / hello 在这里
```

- 用户程序想「打印一个字」「创建任务」，不能直接碰硬件，只能执行 **`ecall`**，掉进内核。  
- 内核通过 **SBI** 再向 OpenSBI 借力（写控制台、设定时器等）。

开机后你在终端里看到的第一大批字，早期往往是 **S-mode 内核**用 SBI 打出来的；之后 shell 的字则是用户态 `SYS_DEBUG_WRITE`。

## 3. DeepRoot 坚持的两件事

### Root Ledger（根账本）

内核里有一个不大的环形缓冲区，记录「启动了」「发生了某种 IPC」等事件。  
它不是区块链，也不是安全审计产品——它是**显微镜**：学习时用来回答「刚才系统里发生了什么」。

代码入口可以先记：`kernel/src/ledger.rs`，启动时在 `kernel_main` 里就会 `LEDGER.record(Boot, …)`。

### Capability Provenance（能力来源）

每张「工作证」不只说你能干什么，还尽量记下**它是怎么被开出来的**（mint / derive …）。  
学权限时，这比只看一个整数 flag 更有教学价值。

## 4. 到 v1.9.0 你到底能得到什么？

已经齐的：

- QEMU 上可启动的 S-mode 内核  
- 物理内存管理 + Sv39  
- 能力与同步 IPC  
- 多用户态 ELF（init / console / ping / shell / hello…）  
- 调度与时钟抢占；**SMP（`-smp 2`）**  
- 交互 shell：**argv / env / history / `&` / `|` / `>`**  
- ramfs + **in-RAM 目录树（mkdir/cd）** + virtio-blk DRFS  
- **自有设备树** + FDT 发现  

刻意没有的（别失望，是范围控制）：

- Linux 应用二进制兼容  
- 完整桌面 / GPU 3D；bash/POSIX 脚本语言  
- 网络、多用户登录；完整 POSIX VFS  

## 5. 这份教程怎么用？

1. 先按 [第一次启动](first-boot.md) 跑通。  
2. 再按 [学习路线图](../path/overview.md) **从 0.1 读到 1.9**。  
3. 想自己加程序 → [动手玩](../hands-on/write-user-prog.md)。

下一章：[你需要准备什么](prerequisites.md)。
