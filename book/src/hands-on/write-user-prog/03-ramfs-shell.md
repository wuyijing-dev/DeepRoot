# 步骤 3：挂进 ramfs 与 shell

这一页只讲：**为什么“程序编出来了”还不等于 shell 能 `run` 它。**

## 1. 挂进 ramfs

你要在 `kernel/src/fs.rs` 里：

- `include_bytes!` 新 ELF
- 在 `FILES` 里加一项

否则 shell 根本找不到这个名字。

## 2. shell 快捷命令是可选项

你可以：

- 只支持 `run echo`
- 或额外加一个 `echo` builtin

教学上推荐先理解 `run path` 这条通用路径，再考虑快捷命令。

## 3. 心智模型

```text
crate 被构建出来
!=
内核知道它
!=
shell 能通过名字执行它
```

下一页：[步骤 4：构建、运行、调试](04-build-debug.md)

