# 版本与标签

## 1. 文档对齐哪一版？

本学习笔记默认对齐 **`v1.7.0`**（SMP 多 hart + 自有 DTS + virtio-blk）。  
页首选择器还可打开 **`v1.6.1`** / **`v1.6.0`** / **`v1.4.1`** / **`v1.4.0`** 等冻结快照。

核对三处：

1. 仓库根 `VERSION` 第一行非注释内容  
2. QEMU 横幅：`DeepRoot microkernel …`  
3. Git 标签（若你按标签检出）：`git checkout v1.7.0`

## 2. 标签怎么用？

```bash
git fetch --tags
git tag -l 'v*'
git checkout v1.7.0
```

冻结教程 HTML 在站点的 `/DeepRoot/<tag>/`（由标签工作流发布）。  
日常开发跟 `main` 时，以 `VERSION` 第一行为准。

## 3. 版本号怎么读？

见根目录 `VERSION` 文件头注释。摘要：

- **MAJOR**：ABI 断裂或平台级跃迁（**2.0** = DT/SMP/显示等集成，不是桌面 OS）  
- **PATCHLEVEL**：一类用户可见能力（0.1 启动、1.2 shell、1.7 SMP…）  
- **SUBLEVEL**：该系列内的修复与打磨  

| PATCHLEVEL | 主题 |
|---|---|
| 0.1–0.6 | 启动 → 调度 |
| 1.0–1.4 | ABI → shell → ramfs → 块替身 |
| 1.5–1.6 | FDT → virtio-blk；1.6.1 自有 DTS |
| **1.7** | 多 hart（SMP） |
| 1.8+ | 更丰富 shell → 简易图形 → 2.0 |

当前 **current** 是 `1.7.0`。下一站见 `VERSION` 的 1.8+。

## 4. 和 GitHub Releases

若维护者打了 GitHub Release，说明文字应指向同一 `VERSION` 与标签。  
学习时以 **git tag + 本站选择器** 为准即可。
