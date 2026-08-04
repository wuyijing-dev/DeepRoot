# 0.1.5 early trap 与 `stvec`

这一页只讲：**为什么系统一开始要先用 early trap，之后再切到用户态 trap。**

## 1. 两个阶段

### early trap

- 还没有用户任务
- 还没有 trap frame 环境
- 目标是“先把错误打印出来”

### user trap

- 已经有当前任务
- 已经能保存/恢复一整套寄存器
- 要承接 ecall、timer interrupt、page fault

## 2. `stvec` 是怎么切换的

1. `trap::init()`：`stvec = early_trap_vector`
2. 用户服务器与调度器准备好后：`trap::enable_user()`
3. `stvec = trap_vector`

## 3. 为什么这对阅读很关键

否则你会把：

- 早期兜底打印逻辑
- 正式的用户态 trap 保存/恢复逻辑

混成一团，看不懂为什么前后会有两套入口。

## 4. 最小排错法

如果你看到问题发生在：

- `trap: early stvec=...` 之前：查 boot/linker/QEMU
- `trap: early stvec=...` 之后但用户态没起来：查 `kernel_main` 顺序
- `trap: user stvec=...` 之后才出问题：查 ecall/timer/fault 分支

下一章：[0.2 内存与页表](../02-mm.md)

