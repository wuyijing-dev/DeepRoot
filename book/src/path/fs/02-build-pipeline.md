# `build.rs` 如何产出嵌入字节

这一页只讲：**ELF 是怎么变成内核里的静态字节的。**

## 1. 谁在编译用户程序？

不是你手动 `cargo build -p deeproot-hello` 再拷进内核。  
`kernel/build.rs` 在编内核时会再拉起 cargo，目标仍是 `riscv64gc-unknown-none-elf`，把产物拷到 `OUT_DIR`。

## 2. 内核怎么引用？

`fs.rs`（以及早期的 `servers.rs` 路径）用：

```text
include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"))
```

编译期把文件内容嵌进内核 ELF。所以：

- 改用户程序 → 通常要触发 `build.rs` 重跑  
- 第一次全量构建会慢，因为要编一串用户包  

## 3. 和 DRFS 的分工（1.4.1）

| | embed（本页） | block DRFS |
|---|---|---|
| 何时写入 | 编译期 | 运行时 `block::init` |
| 典型内容 | ELF + 少量文本 | 教学文本文件 |
| `run` | 可以 | 不可以（今天） |

下一页：[FS_LIST / FS_CAT / EXEC](03-fs-syscalls.md)
