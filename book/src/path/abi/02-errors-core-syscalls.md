# 1.0.2 错误码与核心 syscall

这一页只讲：**新手先该记住哪些 syscall，以及负返回值怎么读。**

## 1. 三个最重要的错误码

| 常量 | 值 | 直觉 |
|---|---|---|
| `ERR_GENERIC` | `-1` | 失败了，但原因要继续往里查 |
| `ERR_AGAIN` | `-11` | 现在还不行，稍后重试 |
| `ERR_NOSYS` | `-38` | syscall 号不存在 / ABI 不匹配 |

## 2. 为什么 `ERR_AGAIN` 不是“坏事”

它经常表示“资源暂时没准备好”：

- shell 读串口，暂时没键
- `wait(id)` 时 child 还没退出
- 某些 IPC 路径里暂时还拿不到结果

因此一看到 `-11`，先别 panic，先问：

```text
是不是这条 syscall 本来就设计成轮询语义？
```

## 3. 新手优先吃透的 5 个 syscall

| syscall | 你在哪里会遇到 | 成功返回 |
|---|---|---|
| `SYS_DEBUG_WRITE` | 所有用户态打印 | 写出长度 |
| `SYS_DEBUG_READ` | shell 输入 | 字节值 |
| `SYS_EXEC` | `run hello` | child sched id |
| `SYS_WAIT` | shell 前台等待 | `0` |
| `SYS_YIELD` | 轮询 / 主动让出 | `0` |

把这 5 个看懂，基本就能读懂 shell 主循环。

## 4. 什么时候查完整 syscall 表

只有当你要读：

- capability 相关路径
- IPC 收发路径
- spawn 与地址空间路径

时，再回去看全表。否则一开始背所有 `SYS_*` 性价比不高。

下一页：[1.0.3 `trap.rs` 如何解码 ecall](03-trap-decode.md)

