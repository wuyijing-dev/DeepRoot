# 1.2.1 shell 主循环

这一页只讲：**shell 到底是怎样一行一行解释命令的。**

## 1. 关键文件

- `user/shell/src/main.rs`

## 2. 主循环骨架

```text
打印 ready
loop:
  打印提示符
  read_line
  trim
  匹配 help/ls/cat/run/...
```

## 3. 为什么这页值得单独拆

因为 shell 的价值不只是“有几个命令”，而是它把：

- 串口输入
- 用户态字符串处理
- syscall 调用
- 前台等待模型

都串成了一条很短、但非常适合新手读的控制流。

下一页：[1.2.2 `read_line` 与共享串口](02-read-line.md)

