# 1.0.3 `trap.rs` 如何解码 ecall

这一页只讲：**内核怎么把一条用户态 `ecall` 识别成具体 syscall。**

## 1. 关键文件

- `kernel/src/trap.rs`
- `kernel/src/sched.rs`

## 2. 跟读 `trap_handler()`

当 trap 来自 U-mode `ecall` 时，核心步骤是：

1. 从当前 trap frame 里取 `nr = tf.x[17]`
2. 取参数 `a0..a3 = tf.x[10..13]`
3. `tf.sepc += 4`
4. 调 `sched::handle_syscall(...)`
5. 把返回值写回发起者
6. `restore_user()` → `sret`

## 3. 为什么 `sepc += 4` 不能漏

因为 `ecall` 本身也是一条指令。  
如果不前移 `sepc`，返回用户态后会再次执行同一条 `ecall`，形成无限 trap。

## 4. 返回值为什么不一定写给“当前正在跑的那个任务”

在简单路径里，两者常常是同一个任务。  
但一旦 syscall 中间触发了阻塞/调度切换，内核必须记住：

```text
这次返回值属于谁发起的 syscall
而不是“最后谁正好在 CPU 上”
```

这也是 `set_syscall_return(issuer, ret)` 的教学意义。

## 5. 最小实验

1. 对照 `trap.rs` 和 `05-abi.md` 当前主章，把 `x17 / x10..13` 再手写一遍。  
2. 自己解释一遍“如果不做 `sepc += 4` 会怎样”。  

下一页：[1.0.4 四个 syscall 实战跟读](04-guided-syscalls.md)

