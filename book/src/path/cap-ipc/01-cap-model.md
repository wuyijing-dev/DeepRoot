# 0.3.1 Capability 模型

这一页只回答一个问题：**DeepRoot 为什么要把权限做成 capability，而不是“我是 root 所以都能做”？**

## 1. 心智模型

把 capability 想成一张由内核签发、放在任务自己口袋里的“票”：

- 票有**类型**：端点、帧、CNode、未类型化内存……
- 票有**权限位**：能不能读、写、grant、做 IPC
- 票有**来源**：它是 mint 出来的还是 derive 出来的

所以 DeepRoot 的常见问题不是“我是哪个 UID”，而是：

1. 我手里有没有这张票？
2. 这张票是不是正确类型？
3. 这张票的 rights 够不够？

## 2. 先看哪些文件

- `kernel/src/cap/`
- `libs/deeproot-abi/src/cap.rs`
- `libs/deeproot-abi/src/rights.rs`

建议你先搜这几个符号：

- `CapType`
- `CapReason`
- `CapSlot`
- `mint_badged`
- `derive`
- `revoke`

## 3. 你要看懂的三个动作

### 3.1 mint

mint 更像“原样签发一张新票”，并可附 badge。

### 3.2 derive

derive 更像“从现有票再裁一张更受限制的票”。

### 3.3 revoke

revoke 不是删一个整数，而是沿着来源关系把一棵子树收回。

这就是为什么文档一直强调 **Capability Provenance**：  
没有来源关系，`revoke subtree` 这件事在教学上就讲不清楚。

## 4. 最小实验

1. 打开 capability 相关 ABI 定义，认清 `CapType` 枚举有哪些值。  
2. 对照 `sched.rs` 中 `SYS_CAP_MINT` / `SYS_CAP_DERIVE` / `SYS_CAP_REVOKE` 分支，看它们最后为什么都回到 `ERR_GENERIC` 或“slot 编号”。  

## 5. 易错点

| 误解 | 正确理解 |
|---|---|
| cap 就是一个数字句柄 | 句柄只是外表，真正关键是类型、rights、来源 |
| revoke 只是删当前 slot | 真正重要的是来源树上的回收范围 |
| 有 slot 就一定能成功 | 还要看 cap 类型和 rights 是否匹配 |

下一页：[0.3.2 启动时的 CSpace 安装](02-boot-cspace.md)

