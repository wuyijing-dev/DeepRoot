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

## 2. 跟读 `_start`（`kernel/src/boot.rs`）

入口汇编做了三件实事：

1. **保存** `a0`/`a1` 到 `s0`/`s1`（清 BSS 会弄脏调用约定用的寄存器）  
2. **`la sp, __boot_stack_top`** — 栈向下增长，符号来自 `kernel/linker.ld`  
3. **把 BSS 段填 0** — C/Rust 的未初始化全局/静态依赖这一点  
4. **`call kernel_main`** — 把 hartid/dtb 放回 `a0`/`a1`

如果 BSS 没清干净，你会遇到「全局变量里是垃圾」这种最难查的鬼畜 bug。所以开机第一课往往是：相信链接脚本 + 清 BSS。

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

## 4. 字是怎么出来的？SBI 控制台

`println!` → `console::_print` → 最终会走到 `sbi::console_putchar` 或批量 `console_write`。

`kernel/src/sbi.rs` 里要点：

- 优先尝试 **Debug Console 扩展**（DBCN）写字节/写缓冲区  
- 失败则回退 **legacy putchar**（EID `0x01`）  
- 读字符用 **legacy getchar**（EID **`0x02`**，千万别和 putchar 弄反——历史上弄反过会导致 shell 读入全是空字节）

对新手：先记住「内核打印 ≈ SBI 借固件写 UART」，用户态打印 ≈ `SYS_DEBUG_WRITE` 再进内核同一套输出路径。

## 5. Trap 为什么这么早初始化？

没有陷阱入口时，一次非法指令/缺页可能直接让你懵在固件里。  
`trap::init` 先挂上向量，后面用户态 ecall、时钟中断才有地方可去。

你在日志里看到的 `trap: early stvec=...` / `trap: user stvec=...`，就是在说：「入口地址已经写进 `stvec`」。

## 6. 动手验证

1. 在 `kernel_main` 横幅前后各加一行独特的 `println!("DBG_A")` / `DBG_B`（本地实验用，勿提交也行）。  
2. 确认顺序符合你的预期。  
3. 故意把 `sbi::console_getchar` 的 EID 改错，跑 shell——观察是否「一回车就 unknown」（学完 shell 章再做）。

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
