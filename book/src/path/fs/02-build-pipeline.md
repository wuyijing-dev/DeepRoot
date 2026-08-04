# 1.3.2 `build.rs` 如何产出嵌入字节

这一页只讲：**为什么改了 `user/hello` 之后，要重编内核才会生效。**

## 1. 构建链

```text
kernel/build.rs
  -> cargo build user/*
  -> 复制 ELF 到 OUT_DIR
fs.rs / servers.rs
  -> include_bytes!(OUT_DIR/...)
```

## 2. 这意味着什么

ramfs 里的 `hello` 不是“运行时从磁盘找到的文件”，而是：

```text
构建期嵌进内核镜像的字节
```

所以你改了用户程序源码，却不重建内核，QEMU 里通常不会变。

## 3. 新手最该记住的一句

在当前教学树里：

```text
改用户程序 ≈ 也要重建内核
```

下一页：[1.3.3 `FS_LIST` / `FS_CAT` / `EXEC`](03-fs-syscalls.md)

