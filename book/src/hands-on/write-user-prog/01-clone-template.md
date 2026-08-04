# 步骤 1：复制 hello 模板

这一页只讲：**为什么写一个新用户程序时，最稳的起点是复制 `user/hello/`。**

## 1. 你会一起拿到什么

- `_start` 汇编骨架
- `#![no_std]` / `#![no_main]`
- `sys::debug_write`
- `sys::exit`
- 链接脚本

这让你不用先自己发明最小运行时。

## 2. 你要先改哪些地方

- `Cargo.toml` 包名
- `[[bin]]` 名字
- `src/main.rs`
- `linker.ld`

## 3. 为什么链接脚本也要看

因为这不是普通 Linux 用户程序。  
它最终会被内核加载到指定虚址，所以基址不是可忽略细节。

下一页：[步骤 2：注册到 workspace 与 build](02-register-build.md)

