# 0.2.1 RAM 发现与 DTB 回退

这一页只讲：**内核怎么知道“哪段物理内存是 RAM，可以拿来用”。**

## 1. 关键文件

- `kernel/src/mm/mod.rs`
- `kernel/src/mm/memmap.rs`

## 2. 思路

优先从 DTB 里拿内存信息；  
如果拿不到，再回退到 QEMU `virt` 的已知默认布局。

## 3. 为什么这一步是整个内存系统的地基

frame allocator、页表页、用户页、内核堆，全都建立在“我先知道哪段 RAM 可用”这个前提上。

## 4. 你该观察什么

启动日志里的 `mm:` 行，通常会告诉你：

- 当前 hart
- RAM 起止
- free 区间

如果这里离谱，后面几乎都会跟着离谱。

下一页：[0.2.2 frame allocator 与 heap](02-frame-heap.md)

