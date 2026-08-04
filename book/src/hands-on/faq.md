# 常见问题（FAQ）

按症状找。仍不解时：把**完整启动日志**和你敲的命令贴出来。

## 构建与环境

### Q: `error: target … may not be installed`

```bash
rustup target add riscv64gc-unknown-none-elf
```

### Q: 找不到 `qemu-system-riscv64`

Debian/Ubuntu：

```bash
sudo apt-get install -y qemu-system-misc
```

确认：`which qemu-system-riscv64`。

### Q: `build-dtb: missing dtc`

```bash
sudo apt-get install -y device-tree-compiler
./scripts/build-dtb.sh
```

详见 [自有设备树](../path/fdt-virtio/01-own-dts.md)。

### Q: 第一次编译特别慢 / 磁盘很大

正常。`kernel/build.rs` 会再编多个 `user/*`，输出在 `target/` 与 `target/user-build/`。

### Q: 我改了 `user/shell`，QEMU 里没变化

1. 确认保存了文件  
2. 再跑 `./scripts/run-qemu.sh`（应触发 kernel rebuild）  
3. 仍不行：`cargo clean -p deeproot-kernel` 后再跑  
4. 确认你没有在跑另一个旧的 QEMU 窗口

### Q: `Failed to get "write" lock`（磁盘镜像）

另一个 QEMU 还占着 `build/deeproot-disk.img`。先关掉旧实例：

```bash
killall qemu-system-riscv64
```

## 启动与 QEMU

### Q: 只有 OpenSBI 横幅，没有 DeepRoot

内核可能没加载成功：检查脚本里 `-kernel` 路径、链接地址、是否编的是 release 目标产物。自定义 `-dtb` 若与硬件严重不符，也可能导致固件/内核起不来——先确认用的是仓库里的 `platform/qemu-virt/deeproot.dts`。

### Q: 有内核横幅，没有 `deeproot>`

往上翻找 `KERNEL PANIC`、`page fault`、`trap`。  
常见：某个用户 ELF 损坏、页表问题、任务立刻 fault。

### Q: 日志里没有 `fdt: model "DeepRoot QEMU virt"`

- 标签是否 ≥ **v1.6.1**  
- `run-qemu.sh` 是否传了 `-dtb build/deeproot-qemu-virt.dtb`  
- `build-dtb.sh` 是否成功  

见 [1.5–1.6 章](../path/10-fdt-virtio.md)。

### Q: 怎么确认是双核在跑？

看内核自己的日志，不要只看 OpenSBI 的 `HART Count`：

- `fdt: cpu count=2`  
- `smp: 2 hart(s) online mask=0x3`  
- `smp: secondary hart=… ready`  
- 两条 `timer: hart=` / `servers: idle hart=`  

详见 [1.7 SMP](../path/11-smp.md)。双核**不等于**性能一定提升（教学负载很轻）。

### Q: `smp: 1 hart(s) online` / 没有 secondary

- `run-qemu.sh` 是否 `-smp 2`  
- DTS 是否有 `cpu@1`（≥ v1.7.0）  
- 日志是否有 `hart_start … failed` / `timed out`  

### Q: `virtio-blk: no block device` / 回退 ramdisk

- QEMU 是否带了 `-device virtio-blk-device,…`  
- DTB 里是否有 `virtio,mmio` 节点  
- MMIO 是否已 `map_mmio_range`（开 Sv39 之后）  

交互仍可能进 shell；`--smoke` 则会失败（要求 virtio 成功）。

### Q: 怎么退出 QEMU？

`Ctrl-A`，松开，再按 `X`。  
无效时另开终端：

```bash
killall qemu-system-riscv64
```

### Q: `--smoke` 失败但交互能进 shell

看 smoke 脚本在 `grep` 哪些字符串；可能日志文案变了。先读 `scripts/run-qemu.sh`（当前应对齐 **1.10.0**）。

## Shell 输入

### Q: 完全不能输入 / 无回显

- 终端焦点是否在 QEMU  
- 是否用了管道/非 TTY 方式启动  
- 是否有大量日志刷屏（仍应能输入，只是难看见）

### Q: 一按回车就 `unknown`，或命令全错

经典内核 bug：`console_getchar` 用了错误的 SBI EID（putchar 的 `0x01` 而不是 getchar 的 `0x02`），或把 legacy 返回值当 modern `{error,value}` 解析。  
对照 `kernel/src/sbi.rs` 与文档 [0.1](../path/01-boot.md) / [1.2](../path/07-shell.md)。

### Q: `run hello` 后提示符再也不回来

hello 若死循环且不 exit，shell 的 `wait` 会一直 `ERR_AGAIN`。重启 QEMU；检查用户程序是否调用了 `sys::exit`。

### Q: `exec failed`

- 名字是否在 `ls` 里  
- 是否对文本文件执行了 `run`  
- ELF 是否过大导致映射失败（看内核是否有相关日志；查 `elf.rs` 的 `MAX_PAGES`）

## 概念澄清

### Q: 这能跑 Linux 程序吗？

不能。ABI 是 DeepRoot-native，不是 Linux syscall。

### Q: ramfs 的文件在磁盘上吗？

**embed ramfs**：不在盘上，是构建期 `include_bytes!` 嵌进内核的。  
**块上 DRFS**（`block.txt` 等）：在 `build/deeproot-disk.img`（经 virtio-blk），见 [1.6](../path/fdt-virtio/03-virtio-blk.md)。

### Q: 设备树是 QEMU 自动生成的吗？

**1.6.1** 起否。真源是 `platform/qemu-virt/deeproot.dts`，经 `dtc` 与 `-dtb` 交给 OpenSBI。见 [自有设备树](../path/fdt-virtio/01-own-dts.md)。

### Q: Bad Apple 算教学内容吗？

不算主线。最多当「大 ELF + 实时 syscall」压力测试。

## 文档站点

### Q: 本地怎么预览学习文档？

```bash
cargo install mdbook
cd book
mdbook serve --open
```

在线：https://wuyijing-dev.github.io/DeepRoot/
