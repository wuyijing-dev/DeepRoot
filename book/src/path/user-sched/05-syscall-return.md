# 0.6.3 syscall 返回值到底写给谁

这一页只讲：**当一次 syscall 中间发生了阻塞/切换，返回值最终该写回哪个任务。**

## 1. 直觉陷阱

很多新手会以为：

```text
handle_syscall 返回了 ret
那就把 ret 写给“当前 CPU 上这个任务”
```

这在最简单路径里常常碰巧没错，但在 IPC / WAIT / 阻塞路径里就不够严谨。

## 2. 正确问题

真正该问的是：

```text
这次 syscall 是谁发起的？
```

因为返回值属于那个发起者，而不是“最后谁正好还在跑”。

## 3. 关键文件

- `kernel/src/trap.rs`
- `kernel/src/sched.rs`

重点跟读：

- `issuer = sched::current_id()`
- `set_syscall_return(issuer, ret)`
- `complete_call(...)`

## 4. 为什么这页很关键

只要把这个点理解了，你就更容易读懂：

- `SYS_WAIT`
- `SYS_IPC_CALL`
- reply 之后 caller 如何恢复

下一章：[1.0 冻结 ABI](../05-abi.md)

