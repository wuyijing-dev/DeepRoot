# 1.14 共享内存 grant（对齐 v1.14.1）

对齐：**v1.14.1**。

## 1.14.0

- `SYS_FRAME_ALLOC` / `MAP` / `MAP_INTO` / `GRANT`（40–43）
- `/grantpeer` 在 `SHARE_VA`（`0x1A00_0000`）读 magic

## 1.14.1

- `SYS_FRAME_UNMAP` / `UNMAP_INTO`（45–46）
- `SYS_CAP_REVOKE` 对 Frame：拆掉已跟踪映射并 `frame::free`
- Root Ledger：`FrameMap` / `FrameUnmap` kind

## 验收

```bash
git checkout v1.14.1
./scripts/run-qemu.sh --smoke
```

应见 `grantpeer: saw magic`、`grant: unmapped`、`grant: revoked frame`、`init: frame revoke ok`。
