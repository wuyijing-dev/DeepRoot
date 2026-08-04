# 1.1.3 跟读 `elf.rs`

这一页只讲：**一个 ELF 字节数组怎样变成可运行任务。**

## 1. 关键文件

- `kernel/src/elf.rs`
- `kernel/src/mm/frame.rs`
- `kernel/src/mm/sv39.rs`

## 2. 你要盯的 5 件事

1. header magic 是否是 `\x7fELF`
2. 是否是 `ET_EXEC`
3. 是否是 `EM_RISCV`
4. `PT_LOAD` 段如何被按页展开
5. 权限位（执行 / 可写）如何映射进页表

## 3. 为什么加载器是新手高难点

因为它把几个层次一次串起来了：

```text
文件格式
-> 段头
-> 虚拟地址范围
-> 物理页分配
-> 页表映射
-> 用户入口 sepc
```

如果你跳过其中任意一层，最后看到的往往只是“进用户态就 page fault”。

## 4. `MAX_PAGES` 为什么重要

教学内核不会无限制加载任意大小 ELF。  
如果程序太大（例如嵌了很多帧数据的用户程序），会因为页数上限被拒绝。

这时 shell 往往只会告诉你：

```text
exec failed
```

真正原因要回到 `elf.rs` 去找。

## 5. 易错点

| 现象 | 根因 |
|---|---|
| bad header | 根本不是 ELF |
| not RISC-V ET_EXEC | 类型或架构不对 |
| 进用户态就 fault | 栈、入口、权限位或映射范围有问题 |

下一页：[1.1.4 Zombie 与 `SYS_WAIT`](04-zombie-wait.md)

