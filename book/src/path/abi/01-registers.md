# 1.0.1 寄存器调用约定

这一页只回答：**用户态到底把 syscall 号和参数塞进哪个寄存器？**

## 1. 最短表格

| 含义 | RISC-V ABI 名字 | trap frame 下标 |
|---|---|---|
| syscall 号 | `a7` | `x17` |
| 参数 0 | `a0` | `x10` |
| 参数 1 | `a1` | `x11` |
| 参数 2 | `a2` | `x12` |
| 参数 3 | `a3` | `x13` |
| 返回值 | `a0` | `x10` |

## 2. 对照哪些源码

- `libs/deeproot-user/src/lib.rs`
- `kernel/src/trap.rs`

## 3. 一次 syscall 的用户态视角

`deeproot-user` 里的统一包装本质上就是：

```text
a7 = SYS_XXX
a0..a3 = 参数
ecall
从 a0 取 ret
```

所以从用户态看，syscall 非常薄：  
它只负责摆好寄存器，然后跳进内核。

## 4. 为什么 trap 里会看到 `tf.x[17]`

因为内核保存的是“整套整数寄存器数组”，而不是 ABI 别名。  
`a7` 只是 `x17` 的另一个名字，所以：

```text
tf.x[17] == a7 == syscall 号
```

## 5. 最小实验

1. 对照 `sys::exec()` 与 `trap_handler()`，确认 path 指针长度分别来自 `a0/a1`。  
2. 自己把一条 syscall 的参数画成寄存器表。  

下一页：[1.0.2 错误码与核心 syscall](02-errors-core-syscalls.md)

