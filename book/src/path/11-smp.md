# 1.7 SMP：多 hart 调度（对齐 v1.7.0）

到 **1.6** 你已经有：自有 DTS、FDT、virtio-blk 上的 DRFS、交互 shell。  
**1.7** 把「单核假设」拆掉——QEMU `-smp 2` 上，两个 hart 都能跑 U-mode 任务。

当前推荐标签：**`v1.7.0`**。

## 本章拆读顺序

1. [HSM 拉起二级核](smp/01-hsm-bringup.md)  
2. [每 hart 运行队列与 idle](smp/02-per-hart-rq.md)  
3. [锁、IPI 与 `tp` 陷阱](smp/03-locks-ipi.md)  

## 1. 这一章要解决什么？

| 误解 | 纠正 |
|---|---|
| 「`-smp 2` 就会自动变快」 | 教学负载很轻；1.7 目标是**正确多核调度**，不是吞吐 |
| 「OpenSBI 会把所有核都送进 `_start`」 | 冷启动通常只有 **一个** boot hart；其余用 **SBI HSM** `hart_start` |
| 「两个核各打各的串口就行」 | `println!` / IPC / 调度表必须 **加锁**；唤醒对端用 **IPI** |
| 「日志里有 OpenSBI HART Count=2 就等于内核双核」 | 还要看内核自己的 `smp: 2 hart(s) online` |

## 2. 一张图串起来

```text
QEMU -smp 2 + deeproot.dts (cpu@0, cpu@1)
        │
        ▼
OpenSBI ──冷启动──► boot hart: _start → kernel_main
        │                │
        │                ├─ mm / satp
        │                ├─ smp::mark_mm_ready
        │                └─ sbi_hart_start(other, _secondary_start)
        │                              │
        │                              ▼
        │                    secondary_main → satp / timer / online
        │
        ▼
servers::bring_up
  · 每 hart 一个 idle
  · 任务 home_hart（如 ping@0, init@1）
  · mark_sched_ready + IPI
        │
        ▼
每 hart：pick_next(home==me) → sret 进 U-mode
跨 hart IPC：Ready 对端任务 → sbi IPI → 对端 WFI 醒来
```

## 3. 版本对照（读日志时用）

| 标签 | 你该看见 |
|---|---|
| ≤1.6.x | `-smp 1`；没有 `smp:` 行 |
| **1.7.0** | `-smp 2`；`fdt: cpu count=2`；`smp: 2 hart(s) online`；两条 `timer: hart=`；两条 `servers: idle hart=` |

## 4. 动手验收（请逐条做）

```bash
git checkout v1.7.0   # 或当前 main，以 VERSION 为准
./scripts/run-qemu.sh --smoke
```

交互启动后，在串口日志里核对：

```text
fdt: cpu count=2
smp: secondary hart=… ready
smp: 2 hart(s) online mask=0x3 …
timer: hart=0 …
timer: hart=1 …          # 顺序可能交错
servers: idle hart=0 …
servers: idle hart=1 …
servers: canopy ready … harts=2
```

`mask=0x3` 表示 hart0 与 hart1 的 online 位都置上了。

## 5. 性能？先别期待

双核**在跑**，不等于 shell / Bad Apple 会明显加速。原因见子页与 FAQ：负载轻、QEMU TCG、共享锁会串行化。1.7 验收的是「两个 hart 都在调度」，不是 benchmark。

## 6. 源码地图

| 文件 | 角色 |
|---|---|
| `kernel/src/smp.rs` | HSM 拉核、online 掩码、IPI、`tp` |
| `kernel/src/boot.rs` | `_start` / `_secondary_start`、每 hart 栈 |
| `kernel/src/sched.rs` | `home_hart`、每 hart current/idle、SCHED_LOCK |
| `kernel/src/sbi.rs` | `hart_start` / `send_ipi_hart` |
| `kernel/src/sync.rs` | 自旋锁 |
| `kernel/src/trap.rs` | 每 hart trap 栈；软中断；CTX_LOCK |
| `platform/qemu-virt/deeproot.dts` | `cpu@0` / `cpu@1` + CLINT/PLIC 接线 |
| `scripts/run-qemu.sh` | `-smp 2` + smoke 字符串 |

下一页：[HSM 拉起二级核](smp/01-hsm-bringup.md)。
