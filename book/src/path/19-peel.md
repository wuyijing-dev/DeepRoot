# 1.14.2 剥离起步：MMIO Frame 与 virtioblk 探测

内核里的完整 virtio-blk 仍服务 DRFS。这一刀只把**设备发现原语**交给 userspace：

1. `SYS_MMIO_VIRTIO(index)` — 按 FDT `virtio-mmio` 下标 mint 一个 `Frame`（badge = MMIO 页 PA）
2. `/virtioblk` — 可加载模块：`FRAME_MAP` 到 `MMIO_VA`（`0x1B00_0000`），读 magic / device_id
3. 找到 `device_id == 2` 时打印 `virtioblk: found block device` / `probe ok`
4. **不**重置 status、不碰 queue — 避免与内核驱动抢设备

## 为何这样剥

| 留在内核 | 移到 userspace |
|---|---|
| 调度 / IPC / 页表 / Cap | 读 MMIO 寄存器、识别设备 |
| 当前 DRFS 用的 virtio-blk I/O | 后续完整 userspace 块驱动（再剥） |

Revoke MMIO Frame 只 unmap，**不会** `frame::free`（设备 PA 不在 RAM 分配器里）。

## 日志要点

```
grant: mmio frame pa=…
virtioblk: probe start
virtioblk: found block device
virtioblk: probe ok
init: virtioblk loaded
```

`./scripts/run-qemu.sh` 的 smoke 已覆盖上述针。

## 下一步

- IRQ / 通知能力交给驱动任务
- 并行 userspace 队列驱动，再切 DRFS 后端
- 勿在内核继续堆 FS 策略（见 VERSION userspace-first）
