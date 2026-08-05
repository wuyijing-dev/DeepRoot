# 1.14 共享内存 grant（对齐 v1.14.0）

对齐：**v1.14.0**。

## 这一站解决什么？

显示合成器需要「一块缓冲两边看」。  
**1.14.0** 先做：分配 Frame 能力（badge = PA）→ 映射进自己的 AS → `FRAME_MAP_INTO` 映射进另一个任务。

- `SYS_FRAME_ALLOC` / `MAP` / `MAP_INTO` / `GRANT`（40–43）
- `SYS_SERVICE_SCHED`（44）按名取 sched id
- 演示：`/grantpeer` 读 `SHARE_VA`（`0x1A00_0000`）上的 magic

## 动手

boot 日志应有：

```text
grant: alloc frame …
grant: mapped into sched=…
grantpeer: saw magic
init: grant peer ok
```

## 验收

```bash
git checkout v1.14.0
./scripts/run-qemu.sh --smoke
```

## 下一小步

- **1.14.1** unmap / revoke + ledger
- **1.15** framebuffer 像素
