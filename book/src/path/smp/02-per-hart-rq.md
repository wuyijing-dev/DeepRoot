# 每 hart 运行队列与 idle

这一页讲：**任务跑在哪个核上、idle 怎么占坑。**

对齐标签：**v1.7.0**。

## 1. 单队列假设被拆掉了

0.6 时代可以想成：

```text
一个 Ready 环 → 一个 current → 一个 idle
```

1.7 变成：

```text
共享 tasks[] 表（加 SCHED_LOCK）
每个 hart：
  current[hart]
  idle_id[hart]
  pick_next：只挑 home_hart == 本 hart 且 Ready 的非 idle 任务
             否则跑本 hart 的 idle
```

每个 `UserTask` 多了字段 **`home_hart`**：它「属于」哪条运行队列。

## 2. 启动时谁挂在哪？

`servers::bring_up`（双核时）大致钉死：

| 任务 | home_hart |
|---|---|
| ping / console | 0 |
| init / shell | 1 |
| idle × N | 各自 hart |

这样 smoke 时两个核都有活干：hart0 跑 ping，hart1 跑 init/shell。  
单核时全部落在 boot hart，行为退回「像以前一样」。

动态 `SYS_SPAWN` / `SYS_EXEC` 会用 `alloc_home_hart()` 在 online hart 间轮转分配。

## 3. 每 hart 一个 idle

`spawn_idle_on(cap, hart)`：

- 每个 idle 有**自己的**用户代码页 / 栈 VA（按 hart 错开），避免共享一页。  
- 用户态死循环：`SYS_YIELD`（ecall）。  
- 若本 hart 没有其它 Ready 任务：内核里 `WFI`，等定时器或 **IPI**。

日志：

```text
servers: idle hart=0 sched_id=…
servers: idle hart=1 sched_id=…
servers: canopy ready … harts=2
```

## 4. `pick_next` / `yield_now` 直觉

对本 hart `h`：

1. 从 `current[h]` 往后扫 `tasks[]`  
2. 找 `Ready && !idle && home_hart == h`  
3. 没有 → 选 `idle_id[h]`  
4. 更新 `current[h]`、切页表、设本 hart 的 `CURRENT_TF[h]`

**不会**从另一 hart 的队列里偷任务（1.7 不做 work-stealing）。跨核协作靠：把任务的 `home_hart` 设对 + IPI 叫醒对端。

## 5. 和 IPC 的关系

某任务在 hart A 上 `recv` 阻塞；hart B 上的调用方 `call` 成功后 `wakeup`：

1. 把 waiter 标成 `Ready`  
2. `smp::ipi_wake(waiter.home_hart)`  

对端若在 idle 的 `WFI` 里，软中断会把它拉起来，再 `yield` 到该任务。

## 6. 动手小实验

1. 在日志里确认 `harts=2` 与两条 idle。  
2. 读 `sched.rs` 里 `home_hart` / `pick_next` / `spawn_idle_on`。  
3. 想一想：若把 shell 的 `home_hart` 也改成 0，hart1 是否几乎只跑 idle？（可改代码试，做完还原。）

下一页：[锁、IPI 与 `tp` 陷阱](03-locks-ipi.md)。
