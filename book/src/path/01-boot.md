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

如果你是第一次读操作系统源码，建议把这里当成“Rust 之前的世界”：

- 还没有堆
- 还没有页表抽象
- 还没有安全检查
- 连普通函数调用都依赖你先把栈准备好

这也是为什么启动代码通常短小、硬核、几乎全是“不能错一步”的机械动作。

## 2. 跟读 `_start`（`kernel/src/boot.rs`）

入口汇编做了三件实事：

1. **保存** `a0`/`a1` 到 `s0`/`s1`（清 BSS 会弄脏调用约定用的寄存器）  
2. **`la sp, __boot_stack_top`** — 栈向下增长，符号来自 `kernel/linker.ld`  
3. **把 BSS 段填 0** — C/Rust 的未初始化全局/静态依赖这一点  
4. **`call kernel_main`** — 把 hartid/dtb 放回 `a0`/`a1`

如果 BSS 没清干净，你会遇到「全局变量里是垃圾」这种最难查的鬼畜 bug。所以开机第一课往往是：相信链接脚本 + 清 BSS。

你可以把这一步当成“把现场整理干净”。后面 `trap`、`mm`、`servers` 之所以能稳定工作，很多初始化都默认“BSS 是 0”。

### 2.1 为什么栈一定要先于 Rust 建好？

因为 `call kernel_main`、函数参数传递、局部变量、返回地址保存，都默认“当前 hart 已经有一块可用栈”。  
如果 `sp` 没设好，哪怕只是一个普通 Rust 函数调用，都可能把数据写到随机地址，结果通常不是“报一个好懂的错”，而是直接死机或诡异 trap。

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

### 3.1 为什么 `servers::bring_up()` 放在最后？

因为它一旦把用户态服务器们装好，并切到调度器入口，系统就从“单线程的启动脚本阶段”进入“多任务动态运行阶段”了。  
在这之后：

- trap 会不时因为 ecall/时钟中断回来
- 调度器会在多个任务之间切换
- 串口输出不再只是“按你写代码的顺序线性出现”

所以在教学上，把它放在最后很重要：前面的步骤都还是“搭舞台”，它才是“演员真的开始演”。

### 3.2 `run-qemu.sh` 其实也属于启动路径的一部分

很多新手只盯着 `kernel/src/`，但真正的“从命令到系统起来”还包括 `scripts/run-qemu.sh`：

1. `cargo build -p deeproot-kernel --release --target riscv64gc-unknown-none-elf`
2. QEMU 选择 `virt` 板型、`rv64` CPU、`-bios default`
3. 把内核 ELF 通过 `-kernel` 交给模拟器
4. `-nographic` 把串口直接接到你的当前终端

这解释了为什么你在本机终端里看到的每一行字符，其实是：

```text
QEMU virt guest
  -> OpenSBI
  -> DeepRoot 内核
  -> SBI 控制台
  -> 主机终端
```

## 4. 字是怎么出来的？SBI 控制台

`println!` → `console::_print` → 最终会走到 `sbi::console_putchar` 或批量 `console_write`。

`kernel/src/sbi.rs` 里要点：

- 优先尝试 **Debug Console 扩展**（DBCN）写字节/写缓冲区  
- 失败则回退 **legacy putchar**（EID `0x01`）  
- 读字符用 **legacy getchar**（EID **`0x02`**，千万别和 putchar 弄反——历史上弄反过会导致 shell 读入全是空字节）

对新手：先记住「内核打印 ≈ SBI 借固件写 UART」，用户态打印 ≈ `SYS_DEBUG_WRITE` 再进内核同一套输出路径。

补一个更“源码向”的理解：`kernel/src/sbi.rs` 里 `console_write` 会优先走 SBI Debug Console（DBCN）批量写；如果固件不支持或失败，就逐字节回退 legacy `console_putchar`。这就是为什么早期输出有时你会看到“先快后慢/先稳后回退”的差异。

### 4.1 `console_putchar` 和 `console_write` 有什么区别？

- `console_putchar`：一次只写 1 个字节，适合最早期、最保守路径
- `console_write`：尝试一次写整个缓冲区，适合 shell 输出、大量文本、ASCII 动画

这不是“写法不同但效果一样”那么简单。  
在 QEMU 里，大量逐字节输出会明显拖慢交互，因此后来的 bulk write 对 shell 体验和 Bad Apple 之类场景都很关键。

## 5. Trap 为什么这么早初始化？

没有陷阱入口时，一次非法指令/缺页可能直接让你懵在固件里。  
`trap::init` 先挂上向量，后面用户态 ecall、时钟中断才有地方可去。

你在日志里看到的 `trap: early stvec=...` / `trap: user stvec=...`，就是在说：「入口地址已经写进 `stvec`」。

### 5.1 `early_trap` 和 `user trap` 的区别

这里很值得你区分两种阶段：

#### 早期 trap

- 还没有用户任务
- 还没有“当前任务 trap frame”
- 更像“出了问题先把现场打印出来”

所以 `early_trap_vector` 的逻辑很简单：拿 `scause/sepc/stval`，调用 `early_trap()` 打印。

#### 用户态 trap

- 已经有当前调度任务
- `sscratch` 里保存着用户 trap frame 指针
- trap 返回前要能恢复用户寄存器并 `sret`

所以真正的 `trap_vector` 会保存大量寄存器，再进 `trap_handler()` 做 ecall / timer / fault 分发。

这两套路径不能混。  
如果在“还没准备 trap frame”的时候就切到用户态 trap 向量，保存/恢复逻辑会直接踩空。

### 5.2 `stvec` 是什么时候从 early 切到 user 的？

启动初期 `trap::init()` 会把 `stvec` 设成 `early_trap_vector`。  
等服务器和调度器准备得差不多后，系统才调用 `trap::enable_user()`，把 `stvec` 改成 `trap_vector`。

这就是日志里你会看到两条不同的 trap 初始化信息的原因：

- `trap: early stvec=...`
- `trap: user stvec=...`

它们不是重复打印，而是在说明：**系统从“只会兜底打印错误”的 trap 阶段，进入了“能承接用户 ecall 和 timer 中断”的 trap 阶段。**

## 6. 动手验证

1. 在 `kernel_main` 横幅前后各加一行独特的 `println!("DBG_A")` / `DBG_B`（本地实验用，勿提交也行）。  
2. 确认顺序符合你的预期。  
3. 故意把 `sbi::console_getchar` 的 EID 改错，跑 shell——观察是否「一回车就 unknown」（学完 shell 章再做）。

额外建议：先别改代码，直接用脚本给的 smoke 目标“对照验收”。`scripts/run-qemu.sh --smoke` 里会检查一组关键字符串（适用于 v1.4.0）：

`DeepRoot microkernel 1.4.0` / `canopy ready` / `ping: pong` / `hello: spawned ELF says hi` / `shell: DeepRoot shell ready` / `block: ramdisk ready` / `init: handing off to shell`。

你可以把这些字符串当成“里程碑坐标”：某一项缺失，通常就意味着启动链路在对应阶段还没成功走完。

### 6.1 我建议你学会的“分段排错法”

如果系统没有完全起来，不要一把抓全部日志。  
按阶段切：

1. **连 OpenSBI 后都没进内核横幅**  
   查 `_start`、链接地址、`-kernel` 是否正确
2. **有横幅，但没 `trap: early stvec` / `mm:`**  
   查 `kernel_main` 顺序、早期 panic
3. **有 `mm:` / `block:`，但没有 `shell:`**  
   查 `servers::bring_up()`、ELF 加载、调度入口
4. **有 `shell:`，但不能输入**  
   查 `console_getchar()` 与 shell 输入循环

## 7. 易错点

| 现象 | 可能原因 |
|---|---|
| 完全无输出 | 内核没进到 `kernel_main`，或 SBI 控制台不可用 |
| 只有 OpenSBI 横幅 | 内核入口地址/链接地址与 OpenSBI 期望不符 |
| 乱码 | 极少见；先查是否混用了错误的串口参数 |
| 早期就 trap 死循环 | `stvec` 未正确设置，或早期访问了未准备好的内存/寄存器环境 |
| 有 shell 但按键怪异 | 往往不是 shell 解析本身，而是 SBI getchar 路径/EID/返回值解析错 |

## 8. 小结

0.1 的学习成果是：你能指着屏幕上的一行字，说出它大概经过  
**Rust 格式化 → 内核 console → SBI → UART → QEMU → 你的终端**。

下一章：[0.2 内存与页表](02-mm.md)。
