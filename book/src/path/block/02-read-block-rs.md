# 1.4.2 跟读 `block.rs`

这一页只讲：**当前块层代码到底已经做到了什么。**

## 1. 关键状态

```text
DISK  : 固定大小内存数组
READY : 是否已初始化
```

## 2. `init`

它做三件事：

1. 向“磁盘”内存写入 banner
2. 标记 ready
3. 打印 `block: ramdisk ready ...`

## 3. `read`

- 未 ready 或越界 -> 返回 0
- 否则从 `DISK[off..]` 拷一段到输出缓冲区

## 4. 为什么这仍然有教学意义

即便它还不是完整 virtio 设备，你也已经能看到：

- block 抽象是什么形状
- 初始化与读取接口在哪里
- 以后替换后端时不一定要改高层 API

下一页：[1.4.3 以后怎么走向 virtio-blk](03-next-step-virtio.md)

