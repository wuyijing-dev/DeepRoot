# 0.6.2 timer / preempt

这一页只讲：**为什么一个死循环用户任务不会永久霸占系统。**

## 1. 关键文件

- `kernel/src/timer.rs`
- `kernel/src/sched.rs`

## 2. 关键路径

```text
timer interrupt
  -> trap_handler
  -> timer::on_interrupt()
  -> sched::preempt()
  -> yield_now()
```

## 3. 为什么这对 shell 生死攸关

没有 timer preemption：

- 一个不主动 `yield` 的用户程序
- 就可能长期占住 CPU
- shell、server、其它任务都没机会恢复运行

## 4. 为什么日志故意安静

源码里已经刻意减少 tick spam。  
原因很现实：串口是共享的，时钟中断若每次都狂打日志，会直接毁掉交互体验。

下一页：[0.6.3 syscall 返回值到底写给谁](05-syscall-return.md)

