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

## 1.2 你可以把 ABI 想成“插头定义”

同样一条 `ecall` 指令，内核之所以知道你是想：

- 打印字符串
- 从 ramfs 执行 ELF
- 查询时间
- 等待子任务结束

不是因为 CPU 懂“高级语义”，而是因为**双方事先约好**：

```text
a7 放 syscall 号
a0..a3 放参数
a0 收返回值
负值表示错误
```

所以 ABI 最怕的不是“代码写得丑”，而是**内核和用户态对同一组寄存器有不同理解**。  
一旦一边觉得 `a0` 是路径指针、另一边把它当 sched id，系统往往不会优雅报错，而是直接跑向 `ERR_GENERIC`、page fault，甚至乱码输出。

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

### 2.1 为什么 DeepRoot 喜欢返回负数？

因为用户态封装统一把 `ecall` 返回值视为 `isize`。  
这让调用者只需要做一个非常朴素的判断：

```text
ret >= 0   → 成功，正数/0 可能自带业务含义
ret < 0    → 出错或暂不可用
```

例如：

- `SYS_DEBUG_WRITE` 成功时返回写出的长度
- `SYS_EXEC` 成功时返回 child sched id
- `SYS_WAIT` 成功回收时返回 `0`
- `SYS_DEBUG_READ` 暂时没键时返回 `-11`

这对新手很友好，因为它把 syscall 学习门槛压到了“先区分正负号”。

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

### 3.3 先记住 5 个最常用 syscall

如果你是第一次读 DeepRoot，不要一上来背完整 syscall 表。  
先把下面 5 个吃透，基本就能走完整条用户态交互链：

| 名字 | 你在哪最容易遇到它 | 成功返回 | 常见失败/特殊值 |
|---|---|---|---|
| `SYS_DEBUG_WRITE` | 所有用户程序打印 | 写出长度 | `-1` 参数异常 |
| `SYS_DEBUG_READ` | shell 读键盘 | 一个字节 | `-11` 暂无输入 |
| `SYS_EXEC` | shell `run xxx` | child sched id | `-1` 找不到/非 ELF/映射失败 |
| `SYS_WAIT` | shell 前台等待子任务 | `0` | `-11` 还没退出 |
| `SYS_YIELD` | 轮询或主动让出 | `0` | 通常不失败 |

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

## 4.1 trap frame 里为什么是 `x[17]` / `x[10]`？

调度器保存的是一个通用寄存器数组 `x[0..31]`。  
RISC-V 的 ABI 名字（`a0`/`a1`/`a7`）只是这些整数寄存器的“别名”：

| ABI 名 | 实际整数寄存器 | trap frame 下标 |
|---|---|---|
| `a0` | `x10` | `tf.x[10]` |
| `a1` | `x11` | `tf.x[11]` |
| `a2` | `x12` | `tf.x[12]` |
| `a3` | `x13` | `tf.x[13]` |
| `a7` | `x17` | `tf.x[17]` |

所以你在 `trap_handler()` 里看到：

```text
nr = tf.x[17]
a0 = tf.x[10]
...
```

并不是“写死魔法数字”，而是在直接用 RISC-V 调用约定读寄存器。

## 4.2 `sepc += 4` 为什么不能少？

`ecall` 本身是一条指令。  
如果 trap 返回前不把 `sepc` 前移 4 字节，CPU 回到用户态后会再次执行**同一条** `ecall`，然后再进 trap，形成“无限重复系统调用”。

这就是很多新手在自己写 trap 路径时最容易漏掉的一步。

## 4.3 从 `handle_syscall()` 里跟 4 个真实例子

打开 `kernel/src/sched.rs` 的 `handle_syscall()`，建议按下面四个分支跟一遍：

### 例 1：`SYS_DEBUG_WRITE`

大意是：

```text
读 a0=ptr, a1=len
len 太大 → ERR_GENERIC
把 [ptr, ptr+len) 当用户缓冲区
调用 console::write_bytes(slice)
返回 len
```

这能解释为什么用户程序打印成功时常得到“写出长度”而不是简单的 0。

### 例 2：`SYS_DEBUG_READ`

它不是阻塞等待，而是：

```text
sbi::console_getchar()
有字节 → 返回字节值
没字节 → ERR_AGAIN
```

所以 shell 的读输入循环必须自己处理 `-11` 并 `yield_now()`。

### 例 3：`SYS_EXEC`

它会：

```text
读取 path 指针与长度
校验长度范围
utf8 解析 path
fs::lookup(path)
校验前 4 字节是不是 ELF
spawn_elf_bytes(...)
返回 child sched id
```

这解释了为什么 `run hello` 的失败点其实很多，不只是“文件不存在”：

- 路径长度非法
- 非 UTF-8
- ramfs 查不到
- 文件存在但不是 ELF
- ELF 太大/映射失败

### 例 4：`SYS_WAIT`

这个分支非常值得新手背下来：

```text
child 越界            → ERR_GENERIC
child == Zombie       → 清空槽位并返回 0
child == Empty        → ERR_GENERIC
其他状态（还在跑）    → ERR_AGAIN
```

于是 shell 的前台运行模型就清楚了：`run` 不是靠中断通知，而是靠 `wait -> -11 -> yield -> wait` 轮询收尸。

## 4.4 ABI 与调度为什么绑得这么紧？

很多教材会把“syscall”讲成纯接口，把“scheduler”讲成另一个主题。  
但在 DeepRoot 里，两者是连着的：

- `SYS_EXIT` 会把当前任务标成 `Zombie`
- `SYS_IPC_RECV` 在收不到消息时会阻塞当前任务
- `SYS_WAIT` 读的是调度器里的任务状态
- `SYS_YIELD` 直接推动下一个任务运行

所以当你调一个“看起来只是 ABI 的问题”时，经常真正的根因在调度状态机里。

## 5. 动手验证

1. 在 `deeproot-user` 给 `debug_write` 临时打日志（或在内核 `SYS_DEBUG_WRITE` 分支计数）。  
2. 自己写用户程序调用非法 `a7`，确认得到 `ERR_NOSYS`。  
3. 对比：`SYS_SPAWN(0)` 与 `SYS_EXEC("hello")` 都能跑 hello，但路径不同——一个吃嵌入 blob id，一个吃 ramfs 名字。

### 5.1 建议你亲自跑的最小实验

#### 实验 A：确认 `ERR_AGAIN` 不是“错误”，而是“暂时没准备好”

在 shell 读一行之前，串口通常大部分时间是空的。  
如果你在 `SYS_DEBUG_READ` 分支临时加计数，会看到它反复走到：

```text
None => ERR_AGAIN
```

而 shell 并不会 panic，只是继续 `yield_now()` 再读。

#### 实验 B：让 `SYS_EXEC` 因不同原因失败

分别试：

```text
deeproot> run nosuch
deeproot> cat hello
deeproot> run readme.txt
```

你要能分清：

- `nosuch`：ramfs 查不到
- `readme.txt`：文件存在，但不是 ELF
- `cat hello`：不是 exec 路径，而是 fs 文本显示路径

#### 实验 C：观察 `wait` 收尸

```text
deeproot> run hello
deeproot> run hello
```

如果第二次仍能正常运行，说明第一次的 child 已被 `SYS_WAIT` 回收掉了，而不是永久占着槽位。

## 6. 易错点

- 改了 ABI 却只重编用户或只重编内核 → 必炸。应整仓构建。  
- 把 Linux 的 `write(1, …)` 脑补进来——号码和语义都对不上。
- 看到 `-11` 就当成“失败” → 对 `read` / `wait` 这类接口来说，它常常只是“再等等”。
- 以为 syscall 返回值总是马上写回当前 CPU 正在跑的那个任务 → IPC/阻塞路径下，返回值归属要看“谁发起了那次 syscall”。
- 只记 syscall 名字，不记参数放在哪个寄存器 → 一到 trap frame 就会完全看不懂 `tf.x[10]` / `tf.x[17]` 这些代码。

下一章：[1.1 地址空间与 spawn](06-as-spawn.md)。
