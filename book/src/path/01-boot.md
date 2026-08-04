# 0.1 启动与串口（详细跟读）

本章带你从「按下运行脚本」走到 `kernel_main`，并搞清字是怎么出现在屏幕上的。

## 1. 上电之后谁先跑？

在 QEMU `virt` + `-bios default` 时：

```text
QEMU 载入 OpenSBI 固件
    → OpenSBI 在 M-mode 初始化硬件抽象
    → 跳到内核入口 _start（S-mode），a0=hartid，a1=DTB 物理地址
    → DeepRoot 清 BSS、设栈、调用 kernel_main
```

你**不需要**会写 OpenSBI；只要记住：内核拿到的 `hartid` / `dtb_pa` 来自固件约定。

更具体一点：在 `kernel/src/boot.rs` 里，`_start` 会先把 `a0`（hartid）和 `a1`（DTB 物理地址）临时保存起来，再清 BSS，再把这两个值用 `mv a0, s0` / `mv a1, s1` 交还给 Rust 的 `kernel_main(hartid, dtb_pa)`。

## 2. 跟读 `_start`（`kernel/src/boot.rs`）

入口汇编做了三件实事：

1. **保存** `a0`/`a1` 到 `s0`/`s1`（清 BSS 会弄脏调用约定用的寄存器）  
2. **`la sp, __boot_stack_top`** — 栈向下增长，符号来自 `kernel/linker.ld`  
3. **把 BSS 段填 0** — C/Rust 的未初始化全局/静态依赖这一点  
4. **`call kernel_main`** — 把 hartid/dtb 放回 `a0`/`a1`

如果 BSS 没清干净，你会遇到「全局变量里是垃圾」这种最难查的鬼畜 bug。所以开机第一课往往是：相信链接脚本 + 清 BSS。

你可以把这一步当成“把现场整理干净”。后面 `trap`、`mm`、`servers` 之所以能稳定工作，很多初始化都默认“BSS 是 0”。

## 3. 跟读 `kernel_main`（`kernel/src/main.rs`）

当前启动顺序（读代码时请逐行对）：

```text
ledger::init + Boot 事件
打印横幅（版本号来自 VERSION）
trap::init          ← 设置早期陷阱入口
mm::init            ← 内存与页表
block::init         ← 教学 ramdisk
timer::init         ← 时钟
servers::bring_up   ← 加载用户 ELF 并进入调度（不会返回）
```

`kernel_main` 的返回类型是 `-> !`：正常路径不会回到 `_start` 的 `wfi` 死循环；那只是兜底。

从 `kernel/src/main.rs` 看，启动链路里这几个阶段各自负责一种“系统感”：

- `ledger::init`：给之后的 boot/trap/IPC 记录打底
- `trap::init`：先挂陷阱入口，否则后续 ecall/缺页只能进入固件的“黑盒”
- `mm::init`：准备 Sv39 页表环境（不然用户态 ELF 没法被正确映射）
- `block::init`：1.4 教学块层替身先 ready（便于你区分“模块已加载但暂时没被用”）
- `servers::bring_up`：装入多个 U-mode ELF 并把控制权交给调度器；这一步通常“不返回”

## 4. 字是怎么出来的？SBI 控制台

`println!` → `console::_print` → 最终会走到 `sbi::console_putchar` 或批量 `console_write`。

`kernel/src/sbi.rs` 里要点：

- 优先尝试 **Debug Console 扩展**（DBCN）写字节/写缓冲区  
- 失败则回退 **legacy putchar**（EID `0x01`）  
- 读字符用 **legacy getchar**（EID **`0x02`**，千万别和 putchar 弄反——历史上弄反过会导致 shell 读入全是空字节）

对新手：先记住「内核打印 ≈ SBI 借固件写 UART」，用户态打印 ≈ `SYS_DEBUG_WRITE` 再进内核同一套输出路径。

补一个更“源码向”的理解：`kernel/src/sbi.rs` 里 `console_write` 会优先走 SBI Debug Console（DBCN）批量写；如果固件不支持或失败，就逐字节回退 legacy `console_putchar`。这就是为什么早期输出有时你会看到“先快后慢/先稳后回退”的差异。

## 5. Trap 为什么这么早初始化？

没有陷阱入口时，一次非法指令/缺页可能直接让你懵在固件里。  
`trap::init` 先挂上向量，后面用户态 ecall、时钟中断才有地方可去。

你在日志里看到的 `trap: early stvec=...` / `trap: user stvec=...`，就是在说：「入口地址已经写进 `stvec`」。

## 6. 动手验证

1. 在 `kernel_main` 横幅前后各加一行独特的 `println!("DBG_A")` / `DBG_B`（本地实验用，勿提交也行）。  
2. 确认顺序符合你的预期。  
3. 故意把 `sbi::console_getchar` 的 EID 改错，跑 shell——观察是否「一回车就 unknown」（学完 shell 章再做）。

额外建议：先别改代码，直接用脚本给的 smoke 目标“对照验收”。`scripts/run-qemu.sh --smoke` 里会检查一组关键字符串（适用于 v1.4.0）：

`DeepRoot microkernel 1.4.0` / `canopy ready` / `ping: pong` / `hello: spawned ELF says hi` / `shell: DeepRoot shell ready` / `block: ramdisk ready` / `init: handing off to shell`。

你可以把这些字符串当成“里程碑坐标”：某一项缺失，通常就意味着启动链路在对应阶段还没成功走完。

## 7. 易错点

| 现象 | 可能原因 |
|---|---|
| 完全无输出 | 内核没进到 `kernel_main`，或 SBI 控制台不可用 |
| 只有 OpenSBI 横幅 | 内核入口地址/链接地址与 OpenSBI 期望不符 |
| 乱码 | 极少见；先查是否混用了错误的串口参数 |

## 8. 小结

0.1 的学习成果是：你能指着屏幕上的一行字，说出它大概经过  
**Rust 格式化 → 内核 console → SBI → UART → QEMU → 你的终端**。

下一章：[0.2 内存与页表](02-mm.md)。
