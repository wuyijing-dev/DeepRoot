# 下一步可以看什么

你已经走完 DeepRoot **1.8**（当前推荐 **`v1.8.0`**）：自有 DTS、virtio-blk、SMP、自研 shell。  

按根目录 `VERSION`：**先不急着上 framebuffer**。接下来用 **较长的 1.9.y** 把文件系统做深，再模块加载与其它实用能力；显示推到更后的系列。版本号会**放慢**——同一主题多用 `SUBLEVEL`，少跳 `PATCHLEVEL`。

## 1. 巩固

- 对照 `help` 把 `|` / `>` / `&` / `export` 各试一遍  
- 读 `user/shell` 与 `kernel/src/pipe.rs`  
- 回看 [1.7 SMP](../path/11-smp.md) / [1.8 shell](../path/12-shell18.md)

## 2. 官方下一站（实用优先）

| 系列 | 目标 | 节奏提示 |
|---|---|---|
| **1.9** | 文件系统加深（目录、落盘、fd…） | 长期停在 `1.9.y` |
| **1.10** | 可加载模块 / 动态服务器 | 同样厚系列 |
| **1.11** | 更多实用运行时（工具、I/O 打磨…） | 从 backlog 选做 |
| **1.12** | Framebuffer 简易 UI（**延后**） | 实用主题够用后再开 |
| **2.0** | 集成发布 | 不赶；主题齐了再 MAJOR |

细节与 W1… 列表以仓库根 **`VERSION`** 为准。

## 3. 不建议

- 移植 bash / glibc  
- 为了「版本好看」每个小功能都 bump PATCHLEVEL  
- 在 FS/模块还薄时强行上桌面/GPU  

远程：`git@github.com:wuyijing-dev/DeepRoot.git`。
