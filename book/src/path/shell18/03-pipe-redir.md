# 管道 `|`、重定向 `>` 与内核原语

对齐：**v1.8.0**。

## 1. 内核侧（最小）

| syscall | 作用 |
|---|---|
| `SYS_PIPE` | 建环形缓冲，返回 pipe id |
| `SYS_PIPE_READ` / `WRITE` | 读写字节 |
| `SYS_PIPE_CLOSE` | 销毁 |
| `SYS_TASK_STDOUT` | 把某 sched 任务的 `DEBUG_WRITE` 指到 pipe（或恢复控制台） |
| `SYS_FS_WRITE` | 写/建 scratch 文本，供 `cat`/`ls` |

TCB 字段 `stdout_pipe`：非空则 `SYS_DEBUG_WRITE` 进 pipe，否则进串口。

## 2. `>` 重定向

```text
echo hello > note.txt
cat note.txt
```

builtin 先把输出集到缓冲区，再 `SYS_FS_WRITE`。  
**不能**覆盖嵌入的 ELF 名（如 `hello`）。

## 3. `|` 管道

```text
echo pipe-demo | cat
```

shell 把左侧产出的字节交给右侧（`cat` 无参数时打印 stdin 缓冲）。  

```text
run hello > out.txt
```

对 ELF：建 pipe → `TASK_STDOUT` → wait → 读出 → 可再写入文件。

## 4. 限制（诚实写清）

- 无 `<` 完整实现（解析会拆开，但未接输入重定向语义）。  
- pipe 容量 512B；大数据会截断。  
- 不是 POSIX job control。

## 5. 动手

```text
echo hi > note.txt
cat note.txt
echo a b | cat
ls
```

在 `kernel/src/pipe.rs`、`sched.rs`（`stdout_pipe`）对照实现。

下一站：[Shell 常用命令](../../hands-on/shell-commands.md) 或路线图 **1.9** framebuffer。
