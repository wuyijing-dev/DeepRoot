# 0.5.2 `servers::bring_up` 跟读

这一页只讲：**多个用户服务器是怎样被一口气装起来的。**

## 1. 关键文件

- `kernel/src/servers.rs`
- `kernel/src/elf.rs`
- `kernel/src/sched.rs`

## 2. 最短路径

1. 准备任务表和 endpoint 表
2. 创建 init / ping / console / shell / idle
3. 给 init 安装必要 capability
4. 把嵌入的 ELF 字节喂给加载器
5. 创建 idle
6. 打开 user trap
7. `enter_first(...)`

## 3. 为什么它很值得单独读

因为它把前几章学的东西都串起来了：

- capability
- ELF
- 地址空间
- 调度器
- trap 切换

如果你能独立复述 `bring_up`，说明你已经开始真正理解系统不是“一坨神秘启动代码”。

下一页：[0.6.1 TaskState 与 BlockReason](03-task-states.md)

