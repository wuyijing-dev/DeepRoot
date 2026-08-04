# 1.1.1 per-task 页表

这一页只讲：**为什么 1.1 之后每个任务都该有自己的地址空间。**

## 1. 如果大家共用一张页表，会怎样？

- 隔离弱
- 一个任务乱写更容易影响别的任务
- 调试时很难区分“是谁自己的地址坏了”

因此 1.1 的核心变化不是“多了一个 syscall”，而是：

```text
每个任务有自己的 root_pa
调度切换时要激活它自己的页表根
```

## 2. 关键文件

- `kernel/src/mm/aspace.rs`
- `kernel/src/mm/sv39.rs`
- `kernel/src/sched.rs`

## 3. 你要看懂的最小路径

1. `AddrSpace::create`
2. `UserTask.root_pa`
3. `UserTask::activate_as()`
4. `sv39::activate(...)`

这条链说明：

- 地址空间不是抽象概念
- 它最终要落成“某个物理页表根被写进 CSR 环境”

## 4. 为什么这对后续 `spawn/exec` 很关键

ELF 不是直接“跑字节”，而是要先被映射进当前任务自己的虚拟地址空间。  
没有 per-task 页表，`exec` 只是“看起来像进程”，隔离却讲不圆。

下一页：[1.1.2 `SYS_SPAWN` 控制流](02-sys-spawn.md)

