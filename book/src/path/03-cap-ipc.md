# 0.3–0.4 能力与 IPC（详细跟读）

本章是 DeepRoot 最「微内核味道」的部分：权限用票（capability），协作用消息（IPC），过程可记账（Ledger）。

## 本章拆读顺序

如果你想按“像书一样”的顺序读，请接着看这些子章节：

1. [0.3.1 Capability 模型](cap-ipc/01-cap-model.md)
2. [0.3.2 启动时的 CSpace 安装](cap-ipc/02-boot-cspace.md)
3. [0.4.1 call / recv / reply](cap-ipc/03-ipc-call-flow.md)
4. [0.4.2 Root Ledger 怎么看](cap-ipc/04-ledger.md)
5. [0.4.3 IPC 与调度状态切换](cap-ipc/05-ipc-sched.md)

## 1. Capability：把权限当成票

### 1.1 为什么不用「UID=0 全能」？

教学微内核更想演示：

- 默认什么都不能做  
- 只有拿到某种类型的 cap，才能碰某种对象（端点、帧、未类型化内存…）  
- 票可以派生、可以收回  

你可以在 `kernel/src/cap/` 与 `deeproot-abi` 里看到类型、rights 掩码、provenance 相关结构。

### 1.2 启动时票怎么发到 init 手里？

跟读 `kernel/src/servers.rs` 的 `bring_up`：

1. 创建多个 `TaskId`（init / console / ping / shell / idle）  
2. 为 ping、console 创建 **endpoint**（带 badge）  
3. 往 **init 的 CSpace** 里安装 endpoint 的副本，并带上 IPC 相关 rights  

于是 init 用户态才能 `ipc_call(slot, …)`——slot 是它 CSpace 里的下标，不是「全局随便写的魔法数」那么简单（对初学者可先当成「第 0 个槽是 ping，第 1 个是 console」）。

## 2. 同步 IPC：call / recv / reply

DeepRoot 教学路径使用同步模型（简化理解）：

```text
init                          ping
  │ ipc_call(ping)              │
  │ ──阻塞等待─────────────────►│ ipc_recv
  │                             │ 处理
  │ ◄──────────────────────────│ ipc_reply
  │ 返回                        │
```

打开：

- `user/init/src/main.rs` — `sys::ipc_call(0, 0x5049, 1)`  
- `user/ping/` — online 后 recv/reply  
- `kernel/src/ipc.rs` — 端点队列、错误码  
- `libs/deeproot-user` — ecall 包装  

`0x5049` 这类 label 只是约定好的「消息种类」；可以当成枚举的原始值。

## 3. Root Ledger：把因果记在环上

`kernel_main` 一开始就：

```text
ledger::init();
LEDGER.record(Boot, ...);
```

之后 trap / panic / IPC 路径也会记。  
用户态可通过 `SYS_LEDGER_DUMP`（若 shell 未暴露，可自己写个小程序或在内核临时 dump）观察。

学习用法：

1. 跑一次 boot  
2. dump ledger  
3. 看 Boot / Ipc* 事件顺序是否符合你对 init↔ping 的理解  

## 4. 完整跟一次「ping: pong」

建议你列一个时间线（自己填）：

1. 调度器第一次跑到的任务是谁？（看 `enter_first`）  
2. ping 打印 `server online` 时，它是否已经 block 在 recv？  
3. init 的 call 如何让 ping 就绪？  
4. reply 之后 init 为何能继续 spawn hello？  

对照源码把空填上——这比只看日志有用得多。

### 4.1 内核里阻塞唤醒在哪？

在 `sched.rs` 搜索：

- `block_current_call` / `block_current_ipc`  
- `wakeup_ipc` / `complete_call`  

IPC 不是「拷贝完字符串就返回」那么简单：经常伴随 **Blocked ↔ Ready** 状态切换。  
若服务器从未被调度，客户端会永远卡在 call 上——这就是为什么 0.6 调度必须和 0.4 IPC 一起理解。

### 4.2 用户态包装

`libs/deeproot-user` 里 `ipc_call` / `ipc_recv` / `ipc_reply` 只是把参数塞进 `a0…`、`a7=SYS_*` 再 `ecall`。  
真正策略在内核；用户库尽量薄。

## 5. 动手验证

1. 改 ping 的回复字符串，确认日志变化。  
2. 让 init 对错误的 slot 做 `ipc_call`，观察返回值是否为负（错误码见 `ERR_*`）。  
3. 在 `ipc_call` 成功路径上加 ledger 记录（若尚未有），dump 对比。

## 6. 易错点

| 现象 | 原因 |
|---|---|
| call 一直不返回 | 对方没 recv / badge 不一致 / 调度没跑到服务器 |
| `ERR_AGAIN` | 非阻塞语义或暂时不可用（读串口更常见） |
| 有 cap 仍失败 | rights 不够，或 cap 类型不匹配 |

## 7. 小结

能力回答「**能不能**」；IPC 回答「**怎么协作**」；Ledger 回答「**刚才发生了什么**」。  
三者一起，才是 DeepRoot 要教的内核味道。

下一章：[0.5–0.6 用户态与调度](04-user-sched.md)。
