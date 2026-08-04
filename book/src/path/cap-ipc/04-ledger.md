# 0.4.2 Root Ledger 怎么看

这一页只回答：**系统里刚刚发生了什么，怎么不靠猜？**

## 1. Ledger 的角色

Root Ledger 是一个环形事件账本。  
它不是“日志文件系统”，也不是安全产品；它的教学价值在于：

- 启动时记下 Boot
- trap 时记下 Trap
- capability 操作时记下 mint/derive/revoke
- IPC 相关路径也能留下因果痕迹

所以它像一台“内核示波器”。

## 2. 去哪里看

- `kernel/src/ledger.rs`
- `kernel/src/main.rs`
- `kernel/src/trap.rs`
- `kernel/src/sched.rs`

尤其先确认 `kernel_main` 一开始就做了：

```text
ledger::init()
LEDGER.record(Boot, ...)
```

## 3. 你应该怎么用它

推荐顺序：

1. 先跑一次完整启动
2. 找一个能 dump ledger 的入口
3. 把事件顺序和你脑中的控制流对照

例如你应该能逐渐回答：

- Boot 事件在什么时候记下？
- 第一次 trap 是早期 trap 还是用户 ecall？
- capability mint / derive 发生在谁的路径上？

## 4. 为什么对新手特别有用

因为很多内核现象“看起来像同时发生”：

- shell 提示符出来了
- ping 也打印了
- init 又在做 IPC

但 Ledger 能逼你按顺序看：  
**哪个先发生、哪个后发生、哪个是同一条因果链上的后续动作。**

## 5. 易错点

| 误解 | 纠正 |
|---|---|
| Ledger 是普通 printf 替代品 | 不是，它记录的是结构化事件 |
| 看到一条事件就说明全部成功 | 还要结合后续事件顺序理解 |
| 不会用就删掉 | 它正是这套教程里最值钱的观察工具之一 |

下一页：[0.4.3 IPC 与调度状态切换](05-ipc-sched.md)

