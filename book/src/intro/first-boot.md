# 第一次启动（详细版）

目标：你不但能看到 `deeproot>`，还能**看懂启动日志在说什么**，并知道失败时从哪查。

## 0. 环境确认清单

在仓库根目录外先执行：

```bash
rustc -V
rustup target list --installed | grep riscv64gc-unknown-none-elf
qemu-system-riscv64 --version
git --version
```

缺 target：

```bash
rustup target add riscv64gc-unknown-none-elf
```

缺 QEMU（Debian/Ubuntu 示例）：

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-misc
```

## 1. 拿到代码并对齐文档基线

```bash
git clone git@github.com:wuyijing-dev/DeepRoot.git
cd DeepRoot
git checkout v1.9.0   # 与本教程文字对齐；更早快照用选择器或其它标签
```

## 2. `run-qemu.sh` 实际做了什么？

打开 `scripts/run-qemu.sh` 阅读，顺序是：

1. `./scripts/build-dtb.sh` —— 把 `platform/qemu-virt/deeproot.dts` 编成 DTB  
2. `cargo build -p deeproot-kernel --release --target riscv64gc-unknown-none-elf`  
3. 若无磁盘镜像则创建 `build/deeproot-disk.img`  
4. 调用 `qemu-system-riscv64`，关键参数包括：  
   - `-machine virt` / `-cpu rv64` / `-m 256M` / `-nographic`  
   - `-bios default`：OpenSBI  
   - `-kernel …/deeproot-kernel`  
   - **`-dtb build/deeproot-qemu-virt.dtb`**：DeepRoot **自有**设备树  
   - `-drive` + `-device virtio-blk-device,…`：教学块设备  
   - `-smp 2`：双 hart（1.7）  
   - `-accel tcg,thread=multi`  

交互模式会 `exec` 进 QEMU；`--smoke` 模式会限时跑并 `grep` 关键字符串。

## 3. 构建时还有「隐藏步骤」

内核的 `kernel/build.rs` 会再启动若干次 `cargo`，把：

`init` / `console` / `ping` / `hello` / `shell` / `badapple` …

编成 RISC-V ELF，复制到 `OUT_DIR`，供 `include_bytes!` 嵌入。  
因此：**第一次编译会久一点**，磁盘上还会有 `target/user-build/`。

若你只改了 `user/shell` 却发现 QEMU 里行为没变：先确认是否触发了 `build.rs` 的重编（改 `main.rs` 一般会），或执行一次干净重编。

## 4. 启动日志导读（按出现顺序理解）

下面是「典型」顺序（个别行可能因版本微调）：

### 4.1 OpenSBI 的横幅

你可能看到 OpenSBI 自己的版本、内存域、平台信息。  
这是**固件**，还不是 DeepRoot。可以先略读。

### 4.2 内核横幅

```text
  DeepRoot microkernel 1.9.0
  RISC-V S-mode · capability microkernel
  remote: git@github.com:wuyijing-dev/DeepRoot.git
```

来自 `kernel_main` 里的 `println!`（`kernel/src/main.rs`）。版本号来自仓库根 `VERSION` 第一行。  
若版本不是你以为的那个，检查 `VERSION` 第一行与 `git describe`。

### 4.3 FDT / SMP / mm / block / timer

```text
fdt: blob pa=...
fdt: model "DeepRoot QEMU virt"
fdt: board deeproot,qemu-virt
fdt: cpu count=2
fdt: virtio-mmio count=8
mm: hart=... ram=... free=...
mm: Sv39 identity map active
smp: secondary hart=… ready
smp: 2 hart(s) online mask=0x3 (boot=…)
virtio-blk: ready mmio=... legacy
block: virtio-blk ready size=... files=... (DRFS)
timer: hart=0 …
timer: hart=1 …
servers: idle hart=0 …
servers: idle hart=1 …
```

含义速记：

- **fdt model/board / cpu count**：自有 DTS；**1.7** 起应看到 `cpu count=2`  
- **smp: … online**：二级核已由 HSM 拉起（见 [1.7 SMP](../path/11-smp.md)）  
- **stvec / mm**：陷阱与页表  
- **virtio-blk / block**：真盘上的 DRFS  
- **timer ×2 / idle ×2**：每 hart 各自的时钟与空闲任务  

### 4.4 加载用户 ELF + canopy

```text
servers: canopy ready (ping=… console=… init=… shell=…) harts=2
servers: teaching path 1.1–1.7 …
```

表示多个用户态任务已挂上调度器，且（双核时）两个 hart 都有 idle。

### 4.5 用户态自我介绍

```text
ping: server online
console: server online
init: root server online
shell: DeepRoot shell ready ...
deeproot>
```

之后 init 还可能继续打印 ping/console/hello，再 `handing off to shell`。  
**提示符已经出现时，后面仍可能插入其它任务的日志**——单串口共享，这是正常现象。

## 5. 交互验收（请完整做一遍）

```text
deeproot> help
deeproot> ls
deeproot> cat readme.txt
deeproot> cat version
deeproot> run hello
```

期望：

- `help` 列出命令  
- `ls` 看到 `readme.txt`、`hello`（elf）等  
- `cat` 打出文本  
- `run hello` 后出现 `hello: spawned ELF says hi`，然后提示符回来  

## 6. 退出与残留进程

| 场景 | 做法 |
|---|---|
| 正常离开 QEMU | `Ctrl-A`，松开，再按 `X` |
| 终端假死 / 快捷键无效 | 另开终端 `pkill -f qemu-system-riscv64` |
| 只要测试能否启动 | `./scripts/run-qemu.sh --smoke` |

注意：`pkill qemu-system-riscv`（名字太长且不带 `-f`）在 Linux 上常会匹配失败。

## 7. 失败时怎么缩小范围？

1. **编译失败**：把完整 rustc 错误贴出来；先看是缺 target 还是某个 `user/*` 挂了。  
2. **QEMU 秒退**：在脚本里暂时去掉 `exec` 前的噪声，或手动跑脚本里的 qemu 命令行。  
3. **有横幅无 shell**：看是否 panic；搜 `KERNEL PANIC` / `page fault`。  
4. **有 shell 不能输入**：确认焦点；确认没有另一个 QEMU 占着；确认你没有在管道模式下运行。

更全的条目见 [常见问题](../hands-on/faq.md)。

下一章：[仓库长什么样](repo-map.md)。
