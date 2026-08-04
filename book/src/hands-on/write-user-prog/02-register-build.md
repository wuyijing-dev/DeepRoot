# 步骤 2：注册到 workspace 与 build

这一页只讲：**为什么新用户程序不只是“多一个目录”，还要接入构建链。**

## 1. 两个必须改的地方

- 根 `Cargo.toml` 的 `members`
- `kernel/build.rs` 的用户包构建列表

## 2. 为什么两个地方都要改

### workspace 不加

Cargo 根本不知道这个 crate 是项目成员。

### build.rs 不加

内核不会替你把这个 ELF 构建出来并复制到 `OUT_DIR`，后续 `include_bytes!` 也就无从谈起。

## 3. 最小检查

构建失败时先问：

1. 包名和 `[[bin]]` 名字是不是一致？
2. `build.rs` 要找的产物名是不是对的？

下一页：[步骤 3：挂进 ramfs 与 shell](03-ramfs-shell.md)

