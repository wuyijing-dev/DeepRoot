# 版本与标签

## 1. 文档对齐哪一版？

本学习笔记默认对齐 **`v1.15.2`**（fbmenu 交互 / 栈修复）。  
页首选择器还可打开更早冻结快照（如 `v1.15.1`、`v1.15.0`…）。

核对三处：

1. 仓库根 `VERSION` 第一行非注释内容  
2. QEMU 横幅：`DeepRoot microkernel …`  
3. Git 标签（若你按标签检出）：`git checkout v1.15.2`

## 2. 标签怎么用？

```bash
git fetch --tags
git tag -l 'v*'
git checkout v1.15.2
```

冻结教程 HTML 在站点的 `/DeepRoot/<tag>/`（由标签工作流发布）。  
日常开发跟 `main` 时，以 `VERSION` 第一行为准。

## 3. 版本号怎么读？

见根目录 `VERSION` 文件头注释。摘要：

- **MAJOR**：刻意的平台里程碑（**2.0** = 集成发布，不是桌面 OS）  
- **PATCHLEVEL**：一个**主题系列**（FS、模块…）；**不要**每个小功能都跳一级  
- **SUBLEVEL**：同一主题内的分段落地与打磨（优先多打 `1.9.y`）
- **1.x** 基础设施路线延伸到 **1.50**（对标 Linux 角色，非 ABI）

| PATCHLEVEL | 主题 |
|---|---|
| 0.1–0.6 | 启动 → 调度 |
| 1.0–1.4 | ABI → shell → ramfs → 块替身 |
| 1.5–1.6 | FDT → virtio-blk；1.6.1 自有 DTS |
| 1.7 | 多 hart（SMP） |
| **1.8** | 自研 shell（argv/env/\|/>/&） |
| 1.9 | 文件系统加深（长系列 `1.9.y`） |
| 1.10 | 可加载模块 / 动态服务器（含 userspace 驱动） |
| **1.11** | FS 持久化（DRFS create/append；fd 在 1.11.1） |
| 1.12 | 实用 I/O 工具（sleep/ledger/hexdump；console 延后） |
| **1.13** | 服务命名 / 发现 |
| 1.14 | 共享内存 grant（缓冲基础） |
| 1.15 | Framebuffer 映射（可选 HW；UI 主题至此关闭） |
| 1.16–1.20 | IRQ / 存储 peel / TTY / virtio-net / mmap |
| 1.21–1.30 | 时钟·调度·notify·poll·mount·cache·DMA… |
| 1.31–1.40 | PCI·熵·RTC·SMP·热插拔·MAC/rlimit/cgroup/ns-lite |
| 1.41–1.50 | UDP/TCP lite·日志·profiling·crash·第二平台·ABI·**1.50 里程碑** |
| 2.0–2.2 | 平台集成 → 驱动打磨 → 可观测性 |

当前 **current** 见 `VERSION` 第一行。完整分期与 out-of-scope 以根目录 `VERSION` 为准。

## 4. 和 GitHub Releases

若维护者打了 GitHub Release，说明文字应指向同一 `VERSION` 与标签。  
学习时以 **git tag + 本站选择器** 为准即可。
