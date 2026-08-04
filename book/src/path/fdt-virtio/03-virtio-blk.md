# virtio-blk 与 DRFS 后端

这一页只讲：**块数据如何从 QEMU 磁盘进到 `cat block.txt`。**

对齐：**v1.6.0+**（驱动） / **v1.6.1**（自有 DTB + 日志验收）。

## 1. 分层（请背这张）

```text
shell: cat block.txt
    → SYS_FS_CAT → fs::cat
        → block::lookup  （DRFS：目录 + 文件偏移）
            → block::read
                → virtio_blk::read_bytes   （优先）
                → 或静态 ramdisk           （无设备时回退）
```

**DRFS 布局**仍与 [1.4 章](../09-block.md) 相同（magic `DRFS`、目录项、载荷）。  
变的是：**底层存储**从「内核里的数组」换成「virtio 后面的 raw 镜像」。

## 2. QEMU 怎么挂盘

`scripts/run-qemu.sh`：

```text
build/deeproot-disk.img     # 没有则 dd 出 1MiB
-drive file=…,if=none,id=hd0
-device virtio-blk-device,drive=hd0,bus=virtio-mmio-bus.0
-dtb build/deeproot-qemu-virt.dtb
```

`bus.0` 对应 DTS / 硬件上靠前的 `virtio,mmio` 槽（常见物理基址 `0x10001000`）。

## 3. 跟读 `virtio_blk.rs`（教学范围）

当前 QEMU virt 报告的是 **legacy MMIO（version = 1）**：

1. 在 `fdt` 的 virtio 列表里找 `device_id == 2`（block）  
2. `map_mmio_range` 后读写寄存器  
3. 协商 feature、设置 `GuestPageSize` / `QueueNum` / `QueuePFN`  
4. 用静态对齐的两页做 legacy vring（保证物理连续）  
5. 每次读写一个 **512 字节扇区**；完成靠 **轮询 used 环**（尚未接 PLIC）

失败时：`block::init` 打日志并回退 ramdisk，shell 仍可用（smoke 则要求必须 virtio 成功）。

## 4. `block::init` 做什么

1. `virtio_blk::init()`  
2. 若成功：DRFS 窗口落在盘前 **64KiB**（教学镜像够用）  
3. 若盘上无 `DRFS` magic：格式化并播种 `block.txt` / `from-block` / `blk-version`  
4. 打印类似：

```text
virtio-blk: ready mmio=0x10001000 capacity=2048 sectors … legacy
block: virtio-blk ready size=65536 files=3 (DRFS)
```

## 5. 和 embed ramfs 的边界（再强调）

| | embed `FILES` | 块上 DRFS |
|---|---|---|
| `ls` / `cat` | 有 | 有 |
| `run` / `SYS_EXEC` | ELF | **否** |

## 6. 动手验证

1. `./scripts/run-qemu.sh --smoke` —— 必须出现 `virtio-blk: ready` 与 `block: virtio-blk ready`。  
2. 交互：`cat block.txt`，应提到 virtio / DTS（1.6.1 播种文案）。  
3. 退出 QEMU 后看 `build/deeproot-disk.img`：再次启动应打印 `found existing DRFS`（内容留在镜像里）。  
4. （进阶）暂时去掉 `-device virtio-blk-device`，应回退 `block: ramdisk ready`——证明分层有效。做完请恢复脚本。

## 7. 故意没做的（避免期望膨胀）

- virtio 1.0 modern 传输、PLIC 完成中断  
- 把 ELF 也放到 DRFS 再 `run`  
- 完整文件系统（ext 等）  

下一站建议：[Shell 常用命令](../../hands-on/shell-commands.md)，或路线图里的 **1.7 SMP**（见 `VERSION`）。
