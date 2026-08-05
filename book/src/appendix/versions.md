# 版本与标签

## 1. 文档对齐哪一版？

本学习笔记默认对齐 **`v1.13.0`**（服务名查找）。  
页首选择器还可打开更早冻结快照（如 `v1.11.0`、`v1.10.1`…）。

核对三处：

1. 仓库根 `VERSION` 第一行非注释内容  
2. QEMU 横幅：`DeepRoot microkernel …`  
3. Git 标签（若你按标签检出）：`git checkout v1.13.0`

## 2. 标签怎么用？

```bash
git fetch --tags
git tag -l 'v*'
git checkout v1.13.0
```

冻结教程 HTML 在站点的 `/DeepRoot/<tag>/`（由标签工作流发布）。  
日常开发跟 `main` 时，以 `VERSION` 第一行为准。

## 3. 版本号怎么读？

见根目录 `VERSION` 文件头注释。摘要：

- **MAJOR**：刻意的平台里程碑（**2.0** = 集成发布，不是桌面 OS）  
- **PATCHLEVEL**：一个**主题系列**（FS、模块…）；**不要**每个小功能都跳一级  
- **SUBLEVEL**：同一主题内的分段落地与打磨（优先多打 `1.9.y`）

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
| 1.15 | Framebuffer 像素 |
| 1.16–1.18 | 输入 → output/buffer → 小合成器 |
| 1.19–1.20 | DeepRoot 显示协议 + 教学客户端（Wayland **启发**） |
| 2.0–2.2 | 平台集成 → 图形实验会话 → 协议打磨 |
| **3.0** | 最接近「像用 Wayland」的教学里程碑（仍非 Linux Wayland ABI） |

当前 **current** 见 `VERSION` 第一行。完整分期与 out-of-scope 以根目录 `VERSION` 为准。

## 4. 和 GitHub Releases

若维护者打了 GitHub Release，说明文字应指向同一 `VERSION` 与标签。  
学习时以 **git tag + 本站选择器** 为准即可。
