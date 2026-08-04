# 0.1.3 跟读 `kernel_main`

这一页只讲：**Rust 世界里的启动顺序为什么要这样排。**

## 1. 顺序

```text
ledger::init
trap::init
mm::init
block::init
timer::init
servers::bring_up
```

## 2. 为什么不能乱排

- 没有 `trap::init`，后面 fault/ecall 没地方接
- 没有 `mm::init`，用户 ELF 没法映射
- 没有 `timer::init`，调度无法抢占
- 一旦 `servers::bring_up` 开始，就进入多任务阶段

## 3. `servers::bring_up` 为什么放最后

因为它意味着：

- 用户任务开始存在
- 调度器开始接管
- 后续输出不再只是“线性启动日志”

所以把它放最后，教学上最清晰。

下一页：[0.1.4 SBI 控制台](04-sbi-console.md)

