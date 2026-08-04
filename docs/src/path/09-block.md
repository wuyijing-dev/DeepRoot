# 1.4 块设备教学替身（详细说明）

## 1. 这一章要解决什么心理预期？

很多人看到路线图写「块设备」，期待立刻：

- QEMU `-drive` 挂上镜像  
- virtio-blk 队列、descriptor  
- 可持久化的文件系统  

**1.4.0 教学树先交的是替身**：一块固定大小的内存数组，假装自己是磁盘。  
目的：

1. 在启动路径上**出现块层模块**（`block::init`）  
2. 留下 `read` / `ready` 这种以后可换成真实后端的形状  
3. 不让初学者在学完 shell/ramfs 的同一周里被 virtio 规范淹死  

请先接受这个范围，再读代码——否则你会觉得「文档骗人」。文档没有骗：标题就是 **stand-in（替身）**。

## 2. 跟读 `kernel/src/block.rs`

核心状态：

```text
DISK: [u8; 4096]     // 静态「磁盘」
READY: AtomicBool    // init 成功后为 true
```

### `init`

1. 把一段 banner 字符串拷进 `DISK` 开头  
2. `READY = true`  
3. 打印：

```text
block: ramdisk ready size=4096 (virtio-blk stand-in)
```

你在 [第一次启动](../intro/first-boot.md) 日志里看到的那一行，就是这里。

### `read(off, out)`

- 未 ready 或偏移越界 → 返回 0  
- 否则从 `DISK[off..]` 拷最多 `out.len()` 字节，返回实际长度  

同步、无中断、无 DMA——教学友好。

## 3. 它和 ramfs 的关系（非常重要）

```text
          ┌─────────────┐
 shell ──►│ SYS_FS_*    │──► fs.rs FILES[] （嵌入字节）
          └─────────────┘

          ┌─────────────┐
 boot  ──►│ block::init │──► DISK[4096] （并排存在）
          └─────────────┘
```

**今天 shell 的 `ls` / `cat` / `run` 并不走 `block::read`。**  
两套东西并排：一条是「用户已经能用的路径 API」，一条是「为以后接真磁盘预留的块层」。

读 `VERSION` 里 1.4 一节时，请区分「已落地的替身」和「仍可继续打磨的真实 virtio」。

## 4. 启动顺序里它排在哪？

`kernel_main` 大致：

```text
trap → mm → block::init → timer → servers::bring_up
```

块层在用户服务器起来之前就 ready。即便暂时没人读它，日志也证明模块活着。

## 5. 动手验证

1. 启动后在日志中确认 `block: ramdisk ready size=4096`。  
2. 打开 `block.rs`，把 `DISK_BYTES` 改成 `8192`（本地实验），重编，看日志 size 是否变。  
3. （进阶）在 `kernel_main` 里临时调用 `block::read(0, &mut buf)`，把读到的 banner 再 `println!` 一次——证明 `read` 通路。做完可还原，避免污染主线。

## 6. 以后换成 virtio-blk 时，你希望改哪里？

理想分层（目标形态，不是 1.4.0 必考题）：

```text
fs / 更高层 API
    ↓
block 抽象：read/write(sector)
    ↓
backend：ramdisk  或  virtio-blk
```

今天只有 ramdisk backend。学完 1.4 主线后，若你想继续，可从 QEMU 加 `-drive if=none,file=…` + virtio 设备入手——那是**进阶课题**，见 [下一步](../appendix/next.md)。

## 7. 易错点

| 误解 | 纠正 |
|---|---|
| 「1.4 应该能挂 U 盘」 | 教学树是 ramdisk 替身 |
| 「ramfs 文件在这块 DISK 里」 | 否，文件在 `include_bytes!` |
| 「没有 virtio 就是没完成 1.4」 | 看 `VERSION`：替身即本系列交付 |

## 8. 教学路径到此封顶

恭喜：从串口到页表、能力、IPC、用户态、调度、ABI、spawn、shell、ramfs、块层替身——你走完了 DeepRoot **1.4.0** 主线。

接下来建议：

1. [Shell 常用命令](../hands-on/shell-commands.md) 再练一轮  
2. [自己写一个用户程序](../hands-on/write-user-prog.md)  
3. [常见问题](../hands-on/faq.md) 收藏备用  
