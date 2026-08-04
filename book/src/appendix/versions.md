# 版本与标签

## 1. 文档对齐哪一版？

本学习笔记默认对齐 **`v1.9.1`**（in-RAM VFS 目录 + shell cwd）。  
页首选择器还可打开更早冻结快照（如 `v1.8.0`、`v1.7.0`…）。

核对三处：

1. 仓库根 `VERSION` 第一行非注释内容  
2. QEMU 横幅：`DeepRoot microkernel …`  
3. Git 标签（若你按标签检出）：`git checkout v1.9.1`

## 2. 标签怎么用？

```bash
git fetch --tags
git tag -l 'v*'
git checkout v1.9.1
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
| 1.10 | 可加载模块 / 动态服务器 |
| 1.11 | 实用运行时打磨 |
| 1.12 | Framebuffer（延后） |
| 2.0 | 集成发布（不赶） |

当前 **current** 是 `1.9.1`。系列内继续 FS 深度见 `VERSION`；framebuffer 仍延后。

## 4. 和 GitHub Releases

若维护者打了 GitHub Release，说明文字应指向同一 `VERSION` 与标签。  
学习时以 **git tag + 本站选择器** 为准即可。
