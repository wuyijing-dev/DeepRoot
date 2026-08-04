# 步骤 4：构建、运行、调试

这一页只讲：**从“我已经写完程序了”到“我真的在 QEMU 里运行到了它”之间，还要确认什么。**

## 1. 基本验证顺序

```text
./scripts/run-qemu.sh
deeproot> ls
deeproot> run echo
```

## 2. 如果失败，优先分层检查

### 构建层

- crate 有没有成功编译
- `build.rs` 有没有复制 ELF

### 挂接层

- `fs.rs` 里有没有正确 `include_bytes!`
- `FILES` 名字是否一致

### 运行层

- ELF 是否能被加载
- `main` 是否真的打印并 exit

## 3. 最值得保留的最小调试表

| 现象 | 先查 |
|---|---|
| 找不到 ELF | `build.rs`、workspace、产物名 |
| `exec failed` | ramfs 名字、ELF header、页数上限 |
| 能 run 无输出 | `debug_write`、panic、是否忘记 `exit` |

回到主教程：[自己写一个用户程序](../write-user-prog.md)

