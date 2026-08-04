# 下一步可以看什么

你已经走完 DeepRoot **1.8** 教学主线（当前推荐 **`v1.8.0`**）：自有 DTS、virtio-blk、SMP、以及更完善自研 shell。  
仓库路线图下一站是 **1.9 framebuffer**，再 **2.0** 集成（见根目录 `VERSION`）。

## 1. 巩固

- 对照 `help` 把 `|` / `>` / `&` / `export` 各试一遍  
- 读 `user/shell` 与 `kernel/src/pipe.rs`  
- 回看 [1.7 SMP](../path/11-smp.md) / [1.8 shell](../path/12-shell18.md)

## 2. 官方下一站

| 系列 | 目标 |
|---|---|
| **1.9** | Framebuffer：清屏、画点/矩形、简单菜单或图形终端 |
| **2.0.0** | DT + virtio + SMP + shell + FB 集成发布 |

## 3. 不建议

- 移植 bash / glibc  
- 把 1.8 shell 当成 POSIX 完整实现去扩条件语句  

远程：`git@github.com:wuyijing-dev/DeepRoot.git`。
