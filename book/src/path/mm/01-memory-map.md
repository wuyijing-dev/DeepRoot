# 0.2.1 RAM 发现与 DTB

这一页只讲：**内核怎么知道“哪段物理内存是 RAM，可以拿来用”。**

## 1. 关键文件

- `kernel/src/mm/mod.rs`
- `kernel/src/mm/memmap.rs`
- `kernel/src/fdt.rs`（1.5+：真正的 FDT 遍历）

## 2. 思路

1. 启动时 `fdt::probe(dtb_pa)` 先走完整棵树。  
2. `memmap::discover` 调用 `fdt::memory_reg()` 取 `/memory` 的 `reg`。  
3. 若拿不到或校验失败，再回退到 QEMU `virt` 默认常量。

**1.6.1** 起 `dtb_pa` 通常指向 **DeepRoot 自有** `deeproot-qemu-virt.dtb`（见 [自有设备树](../fdt-virtio/01-own-dts.md)），不只是 QEMU 临时生成的 blob。

## 3. 为什么这一步是整个内存系统的地基

frame allocator、页表页、用户页、内核堆，全都建立在“我先知道哪段 RAM 可用”这个前提上。

## 4. 你该观察什么

```text
fdt: memory 0x80000000..0x90000000 (256 MiB)
mm: hart=0 dtb=… ram=0x80000000..0x90000000 free=…
```

如果这里离谱，后面几乎都会跟着离谱。

下一页：[0.2.2 frame allocator 与 heap](02-frame-heap.md)
