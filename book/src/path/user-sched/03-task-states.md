# 0.6.1 TaskState 与 BlockReason

这一页只讲：**调度器眼里的任务到底有哪些状态。**

## 1. `TaskState`

最常见：

- `Empty`
- `Ready`
- `Running`
- `Blocked`
- `Zombie`

## 2. `BlockReason`

对当前教程最重要的是：

- `IpcRecv`
- `IpcCall`

这说明“Blocked”不是一个黑盒，而是“为什么被挡住”也被内核记着。

## 3. 为什么新手必须先看状态机

否则你会把很多现象误解成“系统没反应”：

- shell 没提示符
- IPC 不返回
- 子任务看起来结束了却没被回收

其实这些往往只是任务正处在另一个合法状态里。

下一页：[0.6.2 timer / preempt](04-timer-preempt.md)

