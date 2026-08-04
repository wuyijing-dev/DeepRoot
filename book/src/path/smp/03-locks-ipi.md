# 锁、IPI 与 `tp` 陷阱

这一页讲：**多核下什么必须串行化、怎么叫醒对端、一个差点整机炸的坑。**

对齐标签：**v1.7.0**。

## 1. 哪些地方加了锁？

| 锁 / 临界区 | 保护什么 |
|---|---|
| `SCHED_LOCK`（`sched.rs`） | `tasks[]`、current/idle、状态迁移 |
| `CTX_LOCK`（`trap.rs`） | `TaskTable` + `EndpointTable`（能力 / IPC） |
| `FRAME_LOCK`（`frame.rs`） | 物理页 bitmap |
| `CONSOLE_LOCK`（`console.rs`） | SBI 串口输出，避免两行日志拧成麻花 |

注意：**不要**在持有 `CTX_LOCK` 时 `WFI`。  
idle 的 `SYS_YIELD` 走单独的 `syscall_yield()`，避免「一核睡觉抱着 IPC 锁、另一核永远 call 不进去」。

## 2. IPI：软中断叫醒

SBI IPI 扩展：`sbi::send_ipi_hart(hart)` → 目标 hart 的 **SSIP**。

trap 里 supervisor software interrupt（`scause` 中断码 1）：

1. `clear_ssip`  
2. `yield_now`  
3. `restore_user`

用途：对端任务被 `Ready` 后，若那核正闲在 `WFI`，定时器还没到也能尽快跑起来。

## 3. 每 hart trap 栈

`trap_vector` 存完 TrapFrame 后：

```text
sp = __deeproot_hart_stacks + (tp + 1) << 16
```

两个 hart **绝不能**再共用旧的单一 `__boot_stack_top`，否则嵌套破坏栈帧。

## 4. 关键坑：`sret` 把 `tp` 清零

TrapFrame 里有一份用户 `x[4]`（`tp`）。  
早期 `restore_user` 会 `ld tp, …` 从 TF 恢复——而新建任务的 TF 里 **tp=0**。

后果：

1. hart1 `sret` 后 `tp==0`  
2. 再 trap 时按 `tp` 选栈 → **误用 hart0 的栈**  
3. 双核同时写同一栈 → 随机缺页 / 死机  

1.7 的做法：**U-mode 暂不使用 TLS**；`restore_user` **不恢复 TF 里的 tp**，内核始终把 `tp` 留作 hart id。

读代码时看到注释里的 “Keep kernel hart id in `tp`”，指的就是这件事。

## 5. 性能预期（再强调一次）

| 事实 | 说明 |
|---|---|
| 双核在调度 | `smp: 2 hart(s) online` + 每核 timer/idle |
| 体感不一定更快 | shell / IPC 串行、QEMU TCG、锁竞争 |
| 1.7 验收点 | 正确性与可观察的双 hart 日志，不是 benchmark |

## 6. 动手小实验

1. `./scripts/run-qemu.sh --smoke`，确认含 `smp: 2 hart(s) online` 与 `smp: secondary hart=`。  
2. 在 `sched.rs` 的 `restore_user` 里找到「故意不恢复 tp」的注释。  
3. （高阶，选做）临时改回 `ld tp`，看是否复现崩溃——**做完务必还原**。

## 7. 下一站

- 巩固：回 [第一次启动](../../intro/first-boot.md) 对照 1.7 日志。  
- 下一章：[1.8 更完善自研 shell](../12-shell18.md)。  
- 玩命令：[Shell 常用命令](../../hands-on/shell-commands.md)。
