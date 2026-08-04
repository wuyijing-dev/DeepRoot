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

## 2. 错误码

| 常量 | 值 | 含义 |
|---|---|---|
| `ERR_GENERIC` | -1 | 泛型失败 |
| `ERR_AGAIN` | -11 | 暂时没有（如串口无新字节、wait 子任务还在跑） |
| `ERR_NOSYS` | -38 | 未知 syscall |

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

## 4. 动手验证

1. 在 `deeproot-user` 给 `debug_write` 临时打日志（或在内核 `SYS_DEBUG_WRITE` 分支计数）。  
2. 自己写用户程序调用非法 `a7`，确认得到 `ERR_NOSYS`。  
3. 对比：`SYS_SPAWN(0)` 与 `SYS_EXEC("hello")` 都能跑 hello，但路径不同——一个吃嵌入 blob id，一个吃 ramfs 名字。

## 5. 易错点

- 改了 ABI 却只重编用户或只重编内核 → 必炸。应整仓构建。  
- 把 Linux 的 `write(1, …)` 脑补进来——号码和语义都对不上。

下一章：[1.1 地址空间与 spawn](06-as-spawn.md)。
