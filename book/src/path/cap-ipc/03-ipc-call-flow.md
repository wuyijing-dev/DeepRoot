# 0.4.1 call / recv / reply

这一页只讲：**一条同步 IPC 从 caller 到 server 再回 caller，到底走了哪几步？**

## 1. 先看哪些文件

- `user/init/src/main.rs`
- `user/ping/src/main.rs`
- `kernel/src/ipc.rs`
- `kernel/src/sched.rs`

## 2. 最短时间线

```text
init                         ping
  ipc_call(slot, label, x)
    │
    ├─ trap -> SYS_IPC_CALL
    │
    ├─ 内核把消息送到 endpoint
    ├─ caller 若暂时拿不到 reply，会 Blocked
    │
    └──────────────► ping 的 ipc_recv
                       取到消息
                       处理
                     ipc_reply(...)
    ◄────────────────────────────
  caller 被唤醒
  继续执行
```

## 3. 两个你必须记住的点

### 3.1 `ipc_call` 不是“发完就不管了”

它是同步 IPC。  
caller 通常要么立刻拿到 reply，要么进入等待 reply 的阻塞状态。

### 3.2 `ipc_recv` 也不是忙等

server 没消息时，内核可以把它标成 `Blocked`，等消息来时再唤醒。

## 4. 从 `sched.rs` 看 syscall 分支

读这三个分支时，注意返回值与阻塞切换：

- `SYS_IPC_CALL`
- `SYS_IPC_RECV`
- `SYS_IPC_REPLY`

特别留意这些辅助函数：

- `block_current_call`
- `block_current_ipc`
- `wakeup_ipc`
- `complete_call`

这些函数才是“同步 IPC 为什么会停住又恢复”的关键。

## 5. 最小实验

1. 把 ping 返回的 label 或打印字符串改掉，确认 caller 观察到变化。  
2. 让 init 对一个错误 slot 做 `ipc_call`，观察它得到负返回值。  

## 6. 易错点

| 现象 | 根因 |
|---|---|
| call 一直不返回 | 对方没 recv、对方没被调度、或者 badge 不匹配 |
| server 在线但 caller 仍卡住 | reply 没送回正确 caller |
| 以为 IPC 只是函数调用 | 实际上它伴随任务状态切换 |

下一页：[0.4.2 Root Ledger 怎么看](04-ledger.md)

