# shell 主循环

这一页只讲：`user/shell` 启动后在干什么。

## 1. 就绪

打印类似：

```text
shell: DeepRoot shell ready ...
deeproot>
```

然后进入循环：读一行 → 解析 → 执行 → 再出提示符。

## 2. 命令分发（心智模型）

```text
help / exit     → 纯用户态
ls / cat        → SYS_FS_*
run / hello …   → SYS_EXEC (+ WAIT)
unknown         → 打一行提示
```

没有管道、没有变量、没有通配符——故意保持小。

## 3. 和内核日志抢串口

shell 用 `SYS_DEBUG_WRITE`；内核 `println!` 也走 SBI。  
同一条 UART 上，服务器日志可能插在你的输入中间——这是教学机的常态，不是输入坏了。

下一页：[read_line 与共享串口](02-read-line.md)
