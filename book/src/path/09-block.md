# 1.4 块设备教学替身（详细说明）

## 本章拆读顺序

1. [为什么先做替身](block/01-why-standin.md)
2. [跟读 `block.rs`](block/02-read-block-rs.md)
3. [以后怎么走向 virtio-blk](block/03-next-step-virtio.md)

## 1. 这一章要解决什么心理预期？

很多人看到路线图写「块设备」，期待立刻：

- QEMU `-drive` 挂上镜像  
- virtio-blk 队列、descriptor  
- 可持久化的文件系统  

**教学树先交的是替身**：一块内存数组假装磁盘，上面再放一个极简 **DRFS** 镜像。  
目的：

1. 启动路径上**出现块层模块**（`block::init`）  
2. 留下 `read` / `write` / `lookup`，以后可换真实后端  
3. 从 **1.4.1** 起，shell 的 `ls` / `cat` 已能读到块上的文本文件（`run` 仍只用 embed ELF）  
4. 不在同一周把 virtio 规范整本倒给初学者  

## 2. 跟读 `kernel/src/block.rs`

核心状态：

```text
DISK: [u8; 16KiB]    // 静态「磁盘」
READY: AtomicBool    // init 成功后为 true
```

### 磁盘上的 DRFS 布局

```text
[0..4)   magic b"DRFS"
[4..8)   version u32 LE (=1)
[8..12)  file count u32 LE
[16..)   目录项 × N（每项 48 字节：name[32] + offset/length/flags）
然后是文件载荷
```

### `init`

1. 清零 `DISK`，写入 DRFS 头与目录  
2. 播种 `block.txt` / `from-block` / `blk-version` 等文本  
3. `READY = true`，打印类似：

```text
block: ramdisk ready size=16384 files=3 (DRFS / virtio-blk stand-in)
```

### `read` / `write` / `dirent` / `lookup`

- `read` / `write`：按字节偏移访问 `DISK`  
- `dirent` / `lookup`：解析 DRFS，把文件内容拷进调用方缓冲区  

同步、无中断、无 DMA——教学友好。

## 3. 它和 ramfs 的关系（1.4.1）

```text
          ┌─────────────┐
 shell ──►│ SYS_FS_*    │──► fs.rs
          └──────┬──────┘
                 ├── FILES[] （embed：ELF + 部分文本）
                 └── block::lookup （DRFS 文本）
```

| 操作 | embed ramfs | block DRFS |
|---|---|---|
| `ls` | 列出 | 另起一节 `fs: block /` |
| `cat` | 文本 / ELF 提示 | 文本（如 `cat block.txt`） |
| `run` / `SYS_EXEC` | ELF only | **不**从此加载 |

读 `VERSION` 时：1.4.0 = 替身就位；1.4.1 = 路径 API 真正吃到块上的文件。

## 4. 启动顺序里它排在哪？

```text
trap → mm → block::init → timer → servers::bring_up
```

块层在用户服务器起来之前就 ready，shell 一上来就能 `cat block.txt`。

## 5. 动手验证

1. 日志确认 `block: ramdisk ready` 且含 `DRFS`。  
2. shell：`ls` 应看到 `fs: block /` 下的 `block.txt` 等。  
3. `cat block.txt` / `cat from-block` / `cat blk-version`。  
4. `cat version` 仍来自 embed（应是 `1.4.1`）。  
5. （进阶）改 `DISK_BYTES` 或多种子文件，重编看 `size=` / `files=`。

## 6. 以后换成 virtio-blk 时，你希望改哪里？

```text
fs / 更高层 API
    ↓
block 抽象：read/write + DRFS（或其它布局）
    ↓
backend：ramdisk  或  virtio-blk
```

今天只有 ramdisk backend。真实 virtio 是进阶课题，见 [下一步](../appendix/next.md)。

## 7. 易错点

| 误解 | 纠正 |
|---|---|
| 「1.4 应该能挂 U 盘」 | 仍是 ramdisk；布局已是可迁移的 DRFS |
| 「所有文件都在 DISK 里」 | ELF 仍在 `include_bytes!` |
| 「没有 virtio 就是没完成 1.4」 | 看 `VERSION`：替身 + 块上文本即本系列交付 |

## 8. 教学路径到此（1.4 系列）

恭喜：串口 → 页表 → 能力 → IPC → 用户态 → 调度 → ABI → spawn → shell → ramfs → **块上 DRFS 文本**。

接下来建议：

1. [Shell 常用命令](../hands-on/shell-commands.md)  
2. [自己写一个用户程序](../hands-on/write-user-prog.md)  
3. [常见问题](../hands-on/faq.md)  
