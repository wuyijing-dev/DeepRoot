# 1.0 冻结 ABI（详细表）

ABI = Application Binary Interface：用户程序与内核之间的「接线标准」。  
1.0 把底座钉住；1.1–1.4 只**追加**新号码。

## 1. 调用约定（RISC-V 用户态）

| 寄存器 | 用途 |
|---|---|
| `a7` | syscall 号 `SYS_*` |
| `a0`–`a3` | 参数（不够再扩展，当前够用） |
| `a0` | 返回值（负号常表示 `ERR_*`） |

封装见 `libs/deeproot-user/src/lib.rs` 的 `ecall`。

## 1.1 用户态怎么“发 ecall”（跟源码对齐）

你在 `libs/deeproot-user/src/lib.rs` 里能看到用户态统一封装：

```text
ecall(nr, a0, a1, a2, a3):
  让 a7 = nr
  让 a0..a3 = 参数
  执行 ecall
  从返回寄存器取回 ret（约定为 isize）
```

例如：

- `sys::debug_write(s)`：`SYS_DEBUG_WRITE`，参数放到 `a0=s.as_ptr()`、`a1=s.len()`
- `sys::debug_read_byte()`：`SYS_DEBUG_READ`，参数全 0
- `sys::exec(path)`：`SYS_EXEC`，参数放到 `a0=path.as_ptr()`、`a1=path.len()`

**关键点**：用户态并不知道“内核用哪个寄存器怎么解析”，它只负责把 `a7/a0..a3` 摆好，内核 trap 路径再按约定读出来。

## 2. 错误码

| 常量 | 值 | 含义 |
|---|---|---|
| `ERR_GENERIC` | -1 | 泛型失败 |
| `ERR_AGAIN` | -11 | 暂时没有（如串口无新字节、wait 子任务还在跑） |
| `ERR_NOSYS` | -38 | 未知 syscall |

在学习时你可以用一个经验法则判断“这是策略问题还是数据问题”：

- `ERR_AGAIN`：通常是“还没准备好”，按协议 `yield_now()` / 再试即可（例如 `SYS_DEBUG_READ`、`SYS_WAIT` 的轮询语义）
- `ERR_NOSYS`：ABI 不匹配（syscall 号写错、或用户库/内核没一起重编）
- `ERR_GENERIC`：多半是参数/路径/ELF 映射失败（比如 `SYS_EXEC` 找不到 ramfs 文件，或 ELF magic 不对）

## 3. 系统调用一览（v1.4 教学树）

> 下表以 `libs/deeproot-abi/src/syscall.rs` 为准；学的时候请打开文件核对。

### 3.1 1.0 基线（0–9）

| 号 | 名字 | 参数直觉 | 典型用途 |
|---|---|---|---|
| 0 | `SYS_DEBUG_WRITE` | ptr, len | 调试打印 |
| 1 | `SYS_LEDGER_DUMP` | — | 打印账本 |
| 2 | `SYS_CAP_DERIVE` | … | 派生能力 |
| 3 | `SYS_IPC_CALL` | slot, label, word0 | 同步调用 |
| 4 | `SYS_CAP_REVOKE` | … | 收回 |
| 5 | `SYS_CAP_MINT` | … | 签发 |
| 6 | `SYS_IPC_RECV` | badge | 接收 |
| 7 | `SYS_IPC_REPLY` | badge, label, word0 | 回复 |
| 8 | `SYS_YIELD` | — | 让出 CPU |
| 9 | `SYS_EXIT` | code | 退出并变僵尸 |

### 3.2 1.1–1.4 追加

| 号 | 名字 | 参数直觉 | 系列 |
|---|---|---|---|
| 10 | `SYS_SPAWN` | blob_id（0=hello） | 1.1 |
| 11 | `SYS_DEBUG_READ` | — → 字节或 `-11` | 1.2 |
| 12 | `SYS_FS_LIST` | — | 1.3 |
| 13 | `SYS_FS_CAT` | path_ptr, len | 1.3 |
| 14 | `SYS_EXEC` | path_ptr, len → sched id | 1.3 |
| 15 | `SYS_TIME` | — → 毫秒 | 彩蛋/节奏 |
| 16 | `SYS_WAIT` | sched_id → 0 / `-11` | shell 等待子任务 |

## 4. 从 ecall 到 handle_syscall（跟读 trap.rs）

用户态执行 `ecall` 后，`kernel/src/trap.rs` 会走到 `trap_handler()`。

当它判定这是用户态 ecall（`scause` 的异常码对应）时，会做这些事：

1. 读取 syscall 号：`nr = tf.x[17]`（也就是用户态的 `a7`）
2. 读取参数：`a0=tf.x[10]`、`a1=tf.x[11]`、`a2=tf.x[12]`、`a3=tf.x[13]`
3. `tf.sepc += 4`：跳过 ecall 指令，避免回到同一条 ecall 反复触发
4. 调度器分发：`ret = sched::handle_syscall(&mut ctx.tasks, &mut ctx.eps, nr, a0, a1, a2, a3)`
5. 把返回值写回“发起 syscall 的那个任务”：`sched::set_syscall_return(issuer, ret)`
6. `sched::restore_user()`：恢复用户寄存器并 `sret` 回到用户态

因此你能理解两件事：

- `SYS_WAIT` 结束后，`wait(id)` 返回的值是写回给“调用 wait 的那一个任务”的 `a0`
- 如果你看到返回值异常（比如一直是 `-11`），通常不是用户库错了，而是内核调度/阻塞路径没发生你预期的状态切换

## 5. 动手验证

1. 在 `deeproot-user` 给 `debug_write` 临时打日志（或在内核 `SYS_DEBUG_WRITE` 分支计数）。  
2. 自己写用户程序调用非法 `a7`，确认得到 `ERR_NOSYS`。  
3. 对比：`SYS_SPAWN(0)` 与 `SYS_EXEC("hello")` 都能跑 hello，但路径不同——一个吃嵌入 blob id，一个吃 ramfs 名字。

## 6. 易错点

- 改了 ABI 却只重编用户或只重编内核 → 必炸。应整仓构建。  
- 把 Linux 的 `write(1, …)` 脑补进来——号码和语义都对不上。

下一章：[1.1 地址空间与 spawn](06-as-spawn.md)。
