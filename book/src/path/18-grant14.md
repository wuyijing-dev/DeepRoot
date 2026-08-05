# 1.14 共享内存 grant（对齐 v1.14.3）

对齐：**v1.14.3**（Frame 系列收官）。

## 1.14.0–1.14.1

- `SYS_FRAME_ALLOC` / `MAP` / `MAP_INTO` / `GRANT` / `UNMAP` / revoke

## 1.14.2

- `SYS_MMIO_VIRTIO` + 探测-only `/virtioblk`

## 1.14.3

见 [剥离章](19-peel.md)：`ALLOC_N` / `PHYS` / `IRQ_VIRTIO` + hd1 完整 userspace 驱动。

## 验收

```bash
git checkout v1.14.3
./scripts/run-qemu.sh --smoke
```

应见 `grantpeer: saw magic`、`virtioblk: rw ok`、`init: frame revoke ok`。
