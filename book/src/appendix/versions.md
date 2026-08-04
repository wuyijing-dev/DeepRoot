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

- **MAJOR**：ABI 断裂  
- **PATCHLEVEL**：一个功能系列（文档按这个组织章节）  
- **SUBLEVEL**：系列内修复与打磨  

政策摘要（1.x–1.4）：

- DeepRoot-native ABI，**不是** Linux/POSIX  
- syscall **只增不改号**（在冻结后）  
- 不把 bash 搬进来；shell 保持很小  

## 建议的 Git 用法

```bash
# 跟着当前教程学（可复现）
git fetch --tags
git checkout v1.4.1

# 对照块层替身初版
git checkout v1.4.0

# 看最新开发
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

想对比某一系列引入了什么：`git log v1.4.0..v1.4.1 --oneline`。
