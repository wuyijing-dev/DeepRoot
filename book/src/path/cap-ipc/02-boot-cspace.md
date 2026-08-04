# 0.3.2 启动时的 CSpace 安装

这一页只回答：**init 为什么一开机就能对 ping / console 说话？**

## 1. 关键文件

- `kernel/src/servers.rs`
- `kernel/src/cap/`
- `kernel/src/sched.rs`

## 2. 启动时到底发生了什么

跟读 `servers::bring_up` 时，把注意力放在“谁给谁发票”上：

1. 内核创建多个任务对象：`init`、`console`、`ping`、`shell`、`idle`
2. 内核为 `ping`、`console` 创建 endpoint
3. 内核把这些 endpoint 的副本安装进 `init` 的 CSpace
4. `init` 之后才能在用户态里用“slot 编号”做 `ipc_call`

于是：

```text
slot 0 / slot 1
并不是全局魔法数字
而是 init 自己口袋里的第 0 张 / 第 1 张 capability
```

## 3. 为什么这对新手重要

很多人第一次看 `sys::ipc_call(0, 0x5049, 1)` 会误以为：

- `0` 是“系统保留的 ping 号”

其实不是。  
`0` 的真正含义是：**当前任务 CSpace 里的第 0 个槽位**。  
换一个任务、换一种安装顺序，slot 0 完全可能对应别的对象。

## 4. 你应该顺着源码验证什么

1. 在 `servers.rs` 里找到 endpoint 创建处
2. 找到把 capability 安装给 `init` 的地方
3. 回到 `user/init/src/main.rs`，看 `ipc_call(0, ...)` 为什么会成功

## 5. 易错点

| 症状 | 常见根因 |
|---|---|
| `ipc_call(0, …)` 一直失败 | slot 0 根本不是 endpoint |
| 有 endpoint 仍失败 | rights 不够，或 badge/类型不匹配 |
| init 能说话，别的任务不行 | 因为票只安装给了 init |

下一页：[0.4.1 call / recv / reply](03-ipc-call-flow.md)

