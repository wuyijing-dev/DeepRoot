# 版本与标签

## 文档基线

本学习笔记默认对齐 **`v1.4.1`**（DRFS：shell 可读块上文本）。  
页首选择器还可打开冻结快照 **`v1.4.0`**（块层替身初版）。

核对三处是否一致：

1. 仓库根目录 `VERSION` **第一行**  
2. 启动横幅 `DeepRoot microkernel …`  
3. Git 标签（若你按标签检出）：`git checkout v1.4.1`

若你跟的是 `main` 且已超前，以 `VERSION` 为准，并注意文档可能尚未改写。

## 在线怎么切换版本？

GitHub Pages 上：

- 默认站点：`/DeepRoot/`（通常跟踪 `main` 最新教程）  
- 冻结版：`/DeepRoot/v1.4.1/`、`/DeepRoot/v1.4.0/` …  
- 数据源：根目录 [`versions.json`](https://github.com/wuyijing-dev/DeepRoot/blob/main/docs/versions.json)

打 `v*` 标签时，workflow 会把该 tag 的 mdBook 产物发布到 `docs/<tag>/`。

## 版本号怎么读

见 `VERSION` 文件头部注释：

- **MAJOR**：ABI 断裂或平台级跃迁（**2.0** = DT/SMP/显示等集成，不是桌面 OS）  
- **PATCHLEVEL**：一个功能系列（文档按这个组织章节）  
- **SUBLEVEL**：系列内修复与打磨  

政策摘要（1.x–2.0）：

- DeepRoot-native ABI，**不是** Linux/POSIX  
- syscall **尽量只增不改号**；若必须改约定，写进该系列说明  
- shell **自研**扩展；学习 xv6/BusyBox 结构，**不**移植 bash  
- 路线图规划到 **2.0**（详见 `VERSION` 正文 1.5–2.0 节）

## 前方路线（摘要）

| 标签目标 | 内容 |
|---|---|
| 1.5 | 设备树平台发现 |
| 1.6 | virtio-blk 等块后端打磨 |
| 1.7 | 多 hart（SMP） |
| 1.8 | 更完善的自研 shell（含简单管道） |
| 1.9 | Framebuffer 简易 UI（非桌面） |
| 2.0.0 | 集成发布 |

当前 **current** 仍是 `1.4.1`，直到开始做并发布 1.5。

## 建议的 Git 用法

```bash
# 跟着当前教程学（可复现）
git fetch --tags
git checkout v1.4.1

# 对照块层替身初版
git checkout v1.4.0

# 看最新开发（含 2.0 路线图注释）
git checkout main
```

## 标签清单（教学相关）

仓库应提供（以 GitHub Releases/Tags 实际为准）：

- `v1.0.0` … 冻结 ABI 附近  
- `v1.1.0` … 地址空间 / spawn  
- `v1.2.0` … shell  
- `v1.3.0` … ramfs / exec  
- `v1.4.0` … 块层替身（ramdisk 初版）  
- `v1.4.1` … DRFS 镜像；`ls`/`cat` 可读块上文本  
- （规划）`v1.5.0` … `v2.0.0` — 见上文与 `VERSION`

想对比某一系列引入了什么：`git log v1.4.0..v1.4.1 --oneline`。
