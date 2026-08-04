# 下一步可以看什么

你已经走完 DeepRoot **1.10.1**（当前推荐 **`v1.10.1`**）：自有 DTS、virtio-blk、SMP、自研 shell、VFS 目录、**可加载模块**（含从 VFS 文件加载）。  

按根目录 `VERSION`：先实用路径，再显示栈；**最接近 Wayland 用法**的目标是 **3.0**（DeepRoot 自研协议，不是 libwayland 兼容）。接下来仍先做 **1.10.y**（驱动 caps 等）。

## 1. 巩固

- `cp modnote mynote` / `modload mynote` / `modules`；对照 [1.10 章](../path/14-modules.md)  
- 回看 [1.9 FS](../path/13-fs19.md) 与 [1.8 shell](../path/12-shell18.md)

## 2. 官方下一站（摘要）

| 系列 | 目标 | 节奏提示 |
|---|---|---|
| **1.10.y** | VFS 加载、驱动 caps、更多 demo | 当前系列 |
| **1.11–1.13** | FS 持久化 / 工具 / 服务命名 | 仍无图形 |
| **1.14** | 共享内存 grant | 为缓冲打底 |
| **1.15–1.18** | FB → 输入 → attach → 小合成器 | 显示栈 |
| **1.19–1.20** | Wayland **启发**的协议 + 教学客户端 | 非 Linux ABI |
| **2.0–2.2** | 平台集成 → 图形实验会话 | 不赶 |
| **3.0** | 最接近「像用 Wayland」的教学里程碑 | 仍非 weston/GTK |

## 3. 自己玩

- 再写一个 badge 不同的小服务器，用 `modload PATH 0x….` 加载  
- 给 registry 加简单名字查询  

心态：DeepRoot 是**自研能力微内核**；学别人可以，不要搬 Linux kmod。
