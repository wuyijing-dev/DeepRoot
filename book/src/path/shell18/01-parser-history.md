# 解析器：引号、argv、history

对齐：**v1.8.0**。

## 1. argv 怎么来？

`tokenize` 按空白切开，但识别引号：

```text
echo "hello world"   →  argv = ["echo", "hello world"]
echo 'a b' c         →  ["echo", "a b", "c"]
```

特殊符号 `|` `>` `&` 在**无引号**时作为运算符，不进普通 token（由 `split_pipeline` / `find_redir` 处理）。

## 2. history

- 每条非空命令推进环形缓冲（16 条）。  
- 内建 `history` 打印。  
- 终端 **↑**：发送 CSI `ESC [ A`，`read_line` 把它译成「取出上一条并回显」。

教学点：串口只有字节流；「方向键」= ANSI 转义，不是魔法。

## 3. help

`help` 列出 1.8 命令与运算符摘要——比 1.2 的五六行完整。

## 4. 动手

1. `echo "a b"` 与 `echo a b` 对比。  
2. 连续输入几条命令，按 ↑，再敲 `history`。  
3. 在 `user/shell/src/main.rs` 里搜索 `tokenize` / `History`。
