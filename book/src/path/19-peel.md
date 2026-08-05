# 1.14.3 Frame 收尾：多页 DMA、IRQ mint、userspace 块驱动

对齐：**v1.14.3**（1.14 系列结束；下一主题 1.15 framebuffer）。

## 架构

| 盘 | 谁驱动 | 用途 |
|---|---|---|
| hd0 / `virtio-mmio-bus.0` | 内核 `virtio_blk` | DRFS |
| hd1 / `virtio-mmio-bus.1` | `drivers/virtioblk` → `/virtioblk` | peel 演示读写 |

## 内核原语（1.14.3）

- `SYS_FRAME_ALLOC_N(n)` — 连续 `n` 页 DMA span（badge = base PA）
- `SYS_FRAME_PHYS(slot)` — 返回 PA，填 virtio descriptor
- `SYS_IRQ_VIRTIO(index)` — mint `CapType::Irq`（badge = FDT IRQ；尚无 PLIC wait）
- `SYS_MMIO_VIRTIO` — 仍用于 map MMIO 页

`FRAME_MAP` 会按 span 映射全部页（queue 两页一次 map）。

## userspace `drivers/virtioblk`

1. 扫描 FDT virtio；**跳过**第一个 `device_id==2`（内核占用）
2. 认领第二个块设备；mint IRQ cap → `virtioblk: irq cap`
3. `ALLOC_N(2)` + 数据页；legacy 队列 init；轮询完成
4. 写/读 sector 1 magic → `virtioblk: rw ok` / `probe ok`

## 验收

```bash
git checkout v1.14.3
./scripts/run-qemu.sh --smoke
```

应见 `virtioblk: irq cap`、`virtioblk: rw ok`，且两阶段 DRFS 持久化仍过。

## 非目标（停在 Frame）

- 不把 DRFS 迁到 userspace IPC
- 不做 PLIC 投递 / IRQ wait
- 不开始 1.15 ramfb
