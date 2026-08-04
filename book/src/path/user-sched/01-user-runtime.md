# 0.5.1 用户程序最小骨架

这一页只讲：**DeepRoot 用户程序最小可执行长什么样。**

## 1. 共同骨架

打开任意 `user/*/src/main.rs`，你都会看到：

```text
_start
  -> 清 BSS
  -> call main
  -> SYS_EXIT
  -> 若返回则 wfi 死循环
```

## 2. 为什么没有普通 Rust 运行时

- `#![no_std]`
- `#![no_main]`

这意味着：

- 没有 libc
- 没有 `println!`
- 输出靠 `deeproot_user::sys::debug_write`

## 3. 为什么 `main` 结束要显式 exit

因为内核需要知道任务生命周期。  
如果用户程序自己结束了但不发 `SYS_EXIT`，调度器就难以把它转成 `Zombie` 并等待收尸。

下一页：[0.5.2 `servers::bring_up` 跟读](02-bring-up.md)

