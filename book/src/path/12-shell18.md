# 1.8 更完善自研 shell（对齐 v1.8.0）

到 **1.7** 你已经有双 hart 调度。  
**1.8** 把 `user/shell` 从「几个固定命令」扩成带 **argv / 环境 / history / `&` / `|` / `>`** 的自研 shell——**不是** bash 移植。

当前推荐标签：**`v1.8.0`**。

## 本章拆读顺序

1. [解析器：引号、argv、history](shell18/01-parser-history.md)  
2. [env / cd / 后台 `&`](shell18/02-env-cd-bg.md)  
3. [管道 `|`、重定向 `>` 与内核原语](shell18/03-pipe-redir.md)  

## 1. 这一章要解决什么？

| 误解 | 纠正 |
|---|---|
| 「shell 要兼容 bash」 | DeepRoot-native；无条件分支 / 函数 / 脚本语言 |
| 「`\|` 必须先有完整 VFS」 | 1.8 用**内核字节 pipe** + `DEBUG_WRITE` 重定向即可演示 |
| 「`>` 写进 virtio 磁盘」 | 教学默认写到 **scratch 文本 overlay**（`SYS_FS_WRITE`），`ls`/`cat` 可见 |

## 2. 一张图

```text
deeproot> echo hi > note.txt
        │
        ├─ tokenize / 引号
        ├─ 发现 `>` → 阶段产出写入 scratch
        └─ SYS_FS_WRITE("note.txt", "hi\n")

deeproot> run hello | cat     (概念：ELF stdout → pipe → 下一阶段)
        │
        ├─ SYS_PIPE
        ├─ SYS_EXEC hello + SYS_TASK_STDOUT(child, pipe)
        ├─ wait
        └─ PIPE_READ → 打印 / 交给下一命令
```

## 3. 你该在日志 / 交互里看见

启动：

```text
shell: DeepRoot shell 1.8 ready (help, |, >, &, env, history)
```

交互试：

```text
deeproot> help
deeproot> export MSG=hello
deeproot> echo $MSG
deeproot> echo pipe-demo | cat
deeproot> echo saved > note.txt
deeproot> cat note.txt
deeproot> run hello &
deeproot> history
```

## 4. 验收

```bash
git checkout v1.8.0
./scripts/run-qemu.sh --smoke
```

smoke 会核对 `shell: DeepRoot shell 1.8 ready` 等标记。

## 5. 源码地图

| 文件 | 角色 |
|---|---|
| `user/shell/src/main.rs` | 解析、builtins、管道调度 |
| `kernel/src/pipe.rs` | 字节环形 pipe |
| `kernel/src/fs.rs` | scratch `write_scratch` |
| `libs/deeproot-abi/.../syscall.rs` | `SYS_PIPE`…`SYS_FS_WRITE` |
| `sched.rs` | `stdout_pipe`；`DEBUG_WRITE` 分流 |

下一页：[解析器：引号、argv、history](shell18/01-parser-history.md)。
