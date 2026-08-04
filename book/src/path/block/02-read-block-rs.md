# 跟读 `block.rs`

这一页只讲：**当前块层代码到底已经做到了什么（对齐 1.4.1）。**

## 1. 关键状态

```text
DISK  : [u8; 16KiB] 静态「磁盘」
READY : AtomicBool
```

上面不是随便塞的 banner，而是一份 **DRFS** 镜像（magic `DRFS` + 目录 + 载荷）。

## 2. `init`

1. 清零磁盘，写 magic / version / file count  
2. 写入目录项与种子文件（如 `block.txt`）  
3. 标记 ready，打印 `block: ramdisk ready size=… files=… (DRFS …)`

## 3. `read` / `write`

- 未 ready 或越界 → 返回 0  
- 否则在 `DISK[off..]` 与缓冲区之间拷贝，返回实际长度  

## 4. `dirent` / `lookup`

- `dirent(i)`：解析第 i 个目录项  
- `lookup(name, out)`：按名找文件，把内容拷进 `out`，返回 `(copied, total, is_text)`

`fs::list` / `fs::cat` 会调用这些接口；`fs::lookup`（给 `SYS_EXEC`）仍只查 embed 表。

## 5. 为什么这仍然有教学意义

你已经能看到：

- block 抽象的形状（读写偏移）  
- 极简 on-disk 布局如何支撑「按路径取内容」  
- 换 virtio 后端时，理想上只换 `DISK[]` 的访问方式，不必改 shell 路径习惯  

下一页：[以后怎么走向 virtio-blk](03-next-step-virtio.md)
