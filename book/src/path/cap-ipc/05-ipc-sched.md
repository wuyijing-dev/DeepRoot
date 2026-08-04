# 0.4.3 IPC 与调度状态切换

这一页只讲：**为什么 IPC 一定要和调度器一起读。**

## 1. 关键状态

看 `sched.rs` 里的：

- `TaskState`
- `BlockReason`

对 IPC 最重要的是：

- `Ready`
- `Running`
- `Blocked`

以及两类阻塞原因：

- `IpcRecv`
- `IpcCall`

## 2. 典型切换

### caller 发 `ipc_call`

如果暂时拿不到 reply：

```text
Running -> Blocked(IpcCall)
```

### server 做 `ipc_recv`

如果当前没有消息：

```text
Running -> Blocked(IpcRecv)
```

### 消息到达或 reply 完成

内核再通过唤醒逻辑把任务放回：

```text
Blocked -> Ready -> Running
```

## 3. 为什么“只看 ipc.rs”会看不懂

因为 `ipc.rs` 更像对象与消息队列的地方；  
但“谁停住、谁恢复、返回值写给谁”，这些都落在调度器里。

所以真正的阅读路径应该是：

```text
ipc.rs 看消息对象
sched.rs 看任务状态
trap.rs 看 syscall 入口
```

## 4. 最小实验

1. 让 server 故意不 reply，看 caller 是否会一直卡。  
2. 观察内核里 `block_current_call` / `complete_call` 的配合。  

## 5. 你在调 bug 时要先问自己

1. 是消息根本没送到？
2. 还是消息到了，但任务没被唤醒？
3. 还是任务醒了，但返回值没送回原 caller？

只要把这三问分开，很多“IPC 好玄学”的感觉就会消失。

下一章：[0.5–0.6 用户态与调度](../04-user-sched.md)

