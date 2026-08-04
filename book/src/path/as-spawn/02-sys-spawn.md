# 1.1.2 `SYS_SPAWN` 控制流

这一页只讲：**`sys::spawn(0)` 从用户态到新任务出生，到底走了哪几步。**

## 1. 关键文件

- `libs/deeproot-user/src/lib.rs`
- `kernel/src/trap.rs`
- `kernel/src/sched.rs`
- `kernel/src/servers.rs`

## 2. 时间线

1. 用户态 `sys::spawn(0)`
2. `ecall` 进入内核
3. `trap_handler()` 解析 `SYS_SPAWN`
4. `handle_syscall()` 里的 `SYS_SPAWN` 分支运行
5. 根据 `blob_id` 选择 `servers::HELLO_ELF`
6. 为新任务申请 capability 侧 Task
7. 找空闲调度槽，计算 stack base
8. `spawn_elf_bytes(...)`
9. 返回 child sched id

## 3. 为什么只支持 `blob_id=0`

因为它是教学阶段的最小脚手架。  
`SYS_SPAWN` 更像“硬编码的演示入口”；真正更像操作系统的，是后来的 `SYS_EXEC(path)`。

## 4. 槽位和栈为什么绑在一起

调度器会先找一个空闲 sched 槽位，然后用这个槽位序号计算 spawn 栈的虚址基址。  
这样每个 child 的栈地址区间都比较稳定，方便理解和排错。

## 5. 最小实验

1. 让 init 连续 `spawn(0)` 两次，观察返回的 id 是否不同。  
2. 给非法 `blob_id`，确认返回 `ERR_GENERIC`。  

下一页：[1.1.3 跟读 `elf.rs`](03-elf-loader.md)

