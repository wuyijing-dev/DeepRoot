# 自己写一个用户程序（逐步教程）

目标：新增一个可 `run` 的 ELF，例如打印 `echo: …` 的小程序 `echo`。  
做完后你应彻底理解：**用户包 → build.rs → ramfs → shell**。

## 本章拆读顺序

1. [步骤 1：复制 hello 模板](write-user-prog/01-clone-template.md)
2. [步骤 2：注册到 workspace 与 build](write-user-prog/02-register-build.md)
3. [步骤 3：挂进 ramfs 与 shell](write-user-prog/03-ramfs-shell.md)
4. [步骤 4：构建、运行、调试](write-user-prog/04-build-debug.md)

下面以包名 `deeproot-echo`、ramfs 名 `echo` 为例。

## 0. 前置

- 已能 `./scripts/run-qemu.sh` 并看到 `deeproot>`  
- 读过 [1.3 ramfs](../path/08-fs.md)  
- 建议先看一眼现成的 `user/hello/`（最小模板）

## 1. 复制 hello 当模板

在仓库根目录：

```bash
cp -a user/hello user/echo
```

然后改名字（至少改这些处）：

### 1.1 `user/echo/Cargo.toml`

以 `user/hello/Cargo.toml` 为模板，把名字改成 echo：

```toml
[package]
name = "deeproot-echo"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "deeproot-echo"
path = "src/main.rs"

[dependencies]
deeproot-user = { path = "../../libs/deeproot-user" }
```

### 1.2 `user/echo/src/main.rs`

保留 `_start` 汇编骨架，改 `main`：

```rust
#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("echo: hello from my program\n");
    sys::exit(0);
}
```

### 1.3 链接脚本基址（重要）

打开 `user/echo/linker.ld`。`hello` 使用：

```text
. = 0x14000000;
```

其它用户程序（init / shell / …）各有自己的基址。1.1 起每任务自有页表后，**撞基址不像早期那么致命**，但仍建议给 echo 换一个清晰地址（例如 `0x15000000`），避免和 hello 混淆、方便用 objdump 对照。

## 2. 让内核构建系统编进它

编辑 `kernel/build.rs`，在 `for (pkg, bin, src_dir) in [` 列表中增加一行：

```rust
("deeproot-echo", "deeproot-echo", "user/echo"),
```

并在仓库根目录 `Cargo.toml` 的 `members` 列表中加入 `"user/echo"`（与 `"user/hello"` 并列）。

## 3. 挂进 ramfs

编辑 `kernel/src/fs.rs`：

```rust
static ECHO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-echo"));

// 在 FILES 数组里：
File {
    name: "echo",
    data: ECHO_ELF,
},
```

## 4.（可选）给 shell 加捷径

在 `user/shell/src/main.rs` 的 `help` 和分支里加：

```text
echo  →  run_path(b"echo")
```

不加也可以：`run echo` 就够。

## 5. 编译运行

```bash
./scripts/run-qemu.sh
```

然后：

```text
deeproot> ls
deeproot> run echo
```

期望看到你的那行 `echo: …`，然后提示符回来。

## 6. 调试清单

| 失败 | 检查 |
|---|---|
| `build.rs` panic missing elf | package/bin 名是否与 `Cargo.toml`、`[[bin]]` 一致 |
| `include_bytes!` 编译失败 | OUT_DIR 里有没有 `deeproot-echo`；build.rs 是否编了它 |
| `exec failed` | `FILES` 名字是否叫 `echo`；是否真是 ELF；`MAX_PAGES` 是否够 |
| 能 run 无输出 | 是否真的 `debug_write`；是否提前 panic/exit |
| 改代码无效果 | 是否触发重编；试 `cargo clean -p deeproot-kernel` 后再跑脚本 |

## 7. 进阶练习（选做）

1. 让程序调用 `sys::yield_now()` 若干次再退出。  
2. 调用 `sys::spawn(0)` 再 spawn 一个 hello（观察调度）。  
3. 故意制造缺页（写空指针）——观察内核 trap 日志（可能需重启 QEMU）。

## 8. 你学到了什么？

操作系统里的「一个程序」，在 DeepRoot 教学树里被拆成：

1. 用户态源码 + 链接脚本  
2. 构建系统把它变成字节  
3. 内核（或 FS）保管字节  
4. exec/spawn 映射进地址空间  
5. 调度器给它 CPU  
6. 它用 syscall 说话，用 `exit` 谢幕  

把这六步串起来，比背名词有用得多。
