# HSM 拉起二级核

这一页只讲：**第二个 hart 怎么从「停在 OpenSBI」变成「进 DeepRoot 内核」。**

对齐标签：**v1.7.0**。

## 1. 冷启动只有一个 hart

OpenSBI 日志里会有类似：

```text
Platform HART Count         : 2
Boot HART ID                : 0    # 有时也会是 1
Domain0 Next Address        : 0x80200000
```

含义：

- 机器上有 **2** 个 hart（QEMU `-smp 2` + DTS 里两个 `cpu@`）。  
- **只有 Boot HART** 被送到内核入口 `_start`。  
- 另一个 hart 停在固件里，等 **SBI HSM** `sbi_hart_start`。

所以：看见 `HART Count : 2` **还不等于**内核已经双核调度；那只是固件侧资源。

## 2. DeepRoot 启动顺序（boot hart）

跟读 `kernel_main`（`main.rs`）：

```text
smp::init_boot_hart(hartid)     # tp = hartid；online 位置位
trap::init / fdt::probe / mm::init
map MMIO …
smp::mark_mm_ready()            # 二级核可以开始装 satp
smp::boot_secondaries(cpu_count)
block::init / timer::init
servers::bring_up()
```

`boot_secondaries`（`smp.rs`）对每个 **非 boot** 的 hart：

```text
sbi_hart_start(hartid, _secondary_start, opaque=0)
        │
        ▼  （异步）目标 hart 在 S-mode 从 _secondary_start 执行
等待 online 位；打印 smp: hart N online …
```

SBI 约定：目标 hart 醒来时 `a0=hartid`，`a1=opaque`，`satp=0`，`SIE=0`。

## 3. `_secondary_start` 做什么？

`boot.rs` 汇编：

1. `mv tp, a0` —— **hart id 放进 `tp`**（后面 trap 栈、调度都靠它）  
2. 算本 hart 内核栈：`__deeproot_hart_stacks + (hartid+1)<<16`  
3. `call secondary_main`

`secondary_main`：

1. 等 `MM_READY`（等 boot hart 建好 identity map）  
2. `sv39::activate(kernel_root)`  
3. `trap::init_secondary` + `timer::init` + 打开软中断  
4. `mark_online` → 打印 `smp: secondary hart=N ready`  
5. 等 `SCHED_READY`，然后 `enter_first(本 hart 的 idle)`

栈区在 **BSS 之后**（`linker.ld`），避免清 BSS 时毁掉正在用的栈。

## 4. 为什么 boot hart 不一定是 0？

QEMU/OpenSBI 有时选 **hart 1** 做 Boot HART。  
旧代码若写死「`a0!=0` 就 park」，会整机挂死。

1.7 的 `_start`：**谁冷启动进来谁当 boot hart**，用 `a0` 设 `tp` 和栈；`boot_secondaries` 跳过 `boot_hart()`，去拉其余核。

## 5. 你该在日志里看见什么

成功双核示例（顺序可能交错）：

```text
fdt: cpu count=2
timer: hart=1 slice=…          # 二级核先打 timer 也正常
smp: secondary hart=1 ready
smp: hart 1 online (HSM start ok)
smp: 2 hart(s) online mask=0x3 (boot=0)
timer: hart=0 slice=…
```

失败时常见：

| 日志 | 含义 |
|---|---|
| `smp: 1 hart(s) online mask=0x1` | 没拉起第二核（DTS 只有 1 CPU、HSM 失败、超时） |
| `smp: hart_start(N) failed err=…` | SBI 拒绝（hart 不存在 / 地址非法等） |
| `smp: hart N start timed out` | `hart_start` 返回了但二级核没置 online |

## 6. 动手小实验

1. 跑 `./scripts/run-qemu.sh`，在日志里标出 Boot HART 与 `smp: … (boot=N)` 是否一致。  
2. 读 `platform/qemu-virt/deeproot.dts` 的 `cpu@0` / `cpu@1` 与 CLINT/PLIC 的 `interrupts-extended`。  
3. （可选）临时把 `run-qemu.sh` 改成 `-smp 1`，应看到只有 1 hart online；做完请还原。

下一页：[每 hart 运行队列与 idle](02-per-hart-rq.md)。
