# 你需要准备什么（详细）

## 1. 硬件与操作系统

- 一台能装开发工具的电脑（作者在 **Linux** 上验证）  
- macOS / WSL2 一般也可；细节差异见下文  
- 能访问网络：装 Rust、QEMU、克隆仓库、（可选）预览文档  

不需要真 RISC-V 开发板：默认用 **QEMU 模拟**。

## 2. 必装工具（逐项验收）

### 2.1 Rust（stable）

安装：https://rustup.rs/

验收：

```bash
rustc -V
cargo -V
```

### 2.2 RISC-V bare-metal 目标

```bash
rustup target add riscv64gc-unknown-none-elf
rustup target list --installed | grep riscv64gc-unknown-none-elf
```

没有这一条，内核和用户 ELF 都编不过。

### 2.3 QEMU（RISC-V 系统模拟）

Debian / Ubuntu：

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-misc
qemu-system-riscv64 --version
```

其它发行版包名可能不同，关键词是 **`qemu-system-riscv64`**。

### 2.4 设备树编译器（`dtc`）

**1.6.1** 起启动脚本会编译 DeepRoot 自有 DTS，需要主机上的 `dtc`：

```bash
sudo apt-get install -y device-tree-compiler
dtc --version
./scripts/build-dtb.sh   # 应生成 build/deeproot-qemu-virt.dtb
```

### 2.5 Git

```bash
git --version
```

### 2.6 常见可选工具

| 工具 | 用途 |
|---|---|
| `mdbook` | 本地预览本学习文档 |
| `riscv64-unknown-elf-objdump` | 反汇编用户 ELF（进阶） |
| 编辑器 | VS Code / Cursor / vim 均可 |

安装 mdbook：

```bash
cargo install mdbook
```

## 3. 建议先建立的最小概念

不要求精通。遇到词就查 [名词表](../appendix/glossary.md)。

1. **特权级** M / S / U：谁能碰硬件  
2. **页表**：虚址怎么变成物址  
3. **系统调用 / ecall**：用户程序怎么求内核办事  
4. **ELF**：可执行文件长什么样  

完全不懂也可以先完成 [第一次启动](first-boot.md)，再回头补。

## 4. 平台备注

### Linux（推荐）

按上面 apt/rustup 即可。

### WSL2

- 在 **WSL 内**装 Rust 与 QEMU，不要混用 Windows 版路径乱拷  
- 串口交互在 WSL 终端里进行  

### macOS

- 用 Homebrew 安装 `qemu`  
- Rust 同样用 rustup  
- 路径、SIP 等一般不影响本项目的 QEMU virt 流程  

## 5. 磁盘与时间预期

首次 `./scripts/run-qemu.sh`：

- 会编译内核 + 多个用户包  
- `target/` 可能到数百 MB 量级  
- 视机器而定，几分钟都正常  

## 6. 本地预览本学习文档

```bash
cd book
mdbook serve --open
```

在线：https://wuyijing-dev.github.io/DeepRoot/（仓库 Settings → Pages → **main** 分支 → **/docs** 目录）。

下一章：[第一次启动](first-boot.md)。
