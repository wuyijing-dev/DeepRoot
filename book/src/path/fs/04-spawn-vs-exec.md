# `SYS_SPAWN` vs `SYS_EXEC`

这一页只讲：**两种「跑起来一个 ELF」差在哪。**

## 1. 对照表

| | `SYS_SPAWN` | `SYS_EXEC` |
|---|---|---|
| 参数 | blob id（教学里常见 `0=hello`） | 路径字符串 |
| 字节从哪来 | 内核里写死的映射（如 `HELLO_ELF`） | `fs::lookup`（**仅 embed**） |
| 谁在用 | init 早期演示 | shell `run` / `hello` |
| 扩展方式 | 每加程序改分支 | 往 `FILES` 加一项即可 |

## 2. 为什么 shell 用 EXEC？

因为路径模型更接近「操作系统」：用户输入名字，内核解析，再装载。  
blob id 适合启动早期、还没有路径概念时的脚手架。

## 3. 和块层的边界

`SYS_EXEC` **不会**去 `block::lookup`。  
块上文本用 `cat`；要执行的程序仍须挂进 embed ramfs（见 [自己写用户程序](../../hands-on/write-user-prog.md)）。

下一节建议：[Shell 常用命令](../../hands-on/shell-commands.md)
