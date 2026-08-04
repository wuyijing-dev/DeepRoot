# 0.5–0.6 用户态与调度（详细跟读）

到了这一段，DeepRoot 不再是「内核独唱」，而是一台小剧场：多个 U-mode 演员 + 内核导演（调度器）。

## 1. 用户程序的最小骨架

打开任意 `user/*/src/main.rs`，顶部几乎都有同样的故事：

```text
_start:
  清 BSS
  call main
  a7 = SYS_EXIT, ecall
  万一返回就 wfi 死循环
```

要点：

- `#![no_std]` + `#![no_main]`：没有 Rust 标准运行时  
- 没有 libc `printf`：输出走 `deeproot_user::sys::debug_write`  
- `main` 结束必须 `sys::exit`，否则会掉进汇编里的 exit ecall  

`deeproot-user` 里的 `ecall` 约定：

- `a7` = syscall 号  
- `a0…` = 参数  
- 返回值在 `a0`（封装成 `isize`）

## 2. 内核如何装入这些 ELF？

`servers::bring_up`（`kernel/src/servers.rs`）：

1. 准备 Task 表、Endpoint 表  
2. 给 init 安装 ping/console 的 endpoint cap  
3. 对每个服务器：`include_bytes!(OUT_DIR/…)` → `elf::load_into` → `sched::spawn_as`  
4. 创建 idle  
5. `trap::enable_user` + `sched::enter_first(...)` —— **函数不返回**

链接脚本决定每个 ELF 的加载基址（例如 ping/console/init/shell 各不相同），避免在「早期共享认知」阶段撞车；1.1 起每任务自有页表后，更是各活各的虚址空间。

## 3. 调度器你要认识的状态

在 `sched.rs` 里找 `TaskState`：

| 状态 | 含义 |
|---|---|
| Empty | 槽位空闲 |
| Ready | 可运行 |
| Running | 正在跑 |
| Blocked | 等 IPC 等事件 |
| Zombie | 已退出，等待收尸（`SYS_WAIT`） |

关键 API（名字以代码为准）：

- `yield_now` — 主动让出  
- `preempt` — 时钟中断触发的抢占（日志已刻意安静，避免刷屏破坏 shell）  
- `block_current_*` / `wakeup_*` — 与 IPC 阻塞唤醒配合  

`enter_first` 会选中第一个任务，恢复 trap frame，`sret` 进用户态。

## 4. 时钟从哪来？

`timer.rs`：

- 读 `time` CSR（mtime 影子）  
- 用 SBI TIME 扩展设定下一个 tick  
- `TICKS_PER_SLICE` 决定时间片粗细  

没有时钟，就没有抢占；没有抢占，一个死循环用户程序就能饿死 shell。

## 5. 系统调用入口长什么样？

跟读 `trap.rs` 用户 ecall 路径：

1. 保存用户寄存器到当前任务 trap frame  
2. 读 `a7` 得到 syscall 号  
3. `sched::handle_syscall(...)`  
4. 把返回值写回**发起 syscall 的那个任务**（即使中间发生过调度）  
5. `restore_user`

这能解释：为什么 `SYS_WAIT` / IPC 阻塞实现时要小心「返回值到底写给谁」。

## 6. 动手验证

1. 在 `user/hello` 里多打印两行，`run hello` 看输出顺序。  
2. 在 hello 里加一个死循环（本地玩），观察 shell 是否仍被时钟抢占（应仍能看到其它日志/最终需重启 QEMU）。  
3. 阅读 `SYS_YIELD` 分支：idle 任务在无可运行任务时为何 `wfi`？

## 7. 易错点

| 现象 | 原因 |
|---|---|
| 用户程序一运行就 fault | 栈没映射、入口错、satp 错 |
| 阻塞在 IPC 再也看不见 shell | 服务器没被调度 / 死锁 |
| 修改 user 程序无生效 | 内核未重编嵌入的 ELF |

下一章：[1.0 冻结 ABI](05-abi.md)。
