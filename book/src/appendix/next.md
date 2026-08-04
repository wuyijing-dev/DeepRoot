# 下一步可以看什么

你已经走完 DeepRoot **1.10.0**（当前推荐 **`v1.10.0`**）：自有 DTS、virtio-blk、SMP、自研 shell、VFS 目录、**可加载模块**。  

按根目录 `VERSION`：framebuffer 仍延后。接下来可在 **1.10.y** 加更多 demo / unload，或开 **1.11** 实用运行时。版本号继续放慢。

## 1. 巩固

- `modload moddemo` / `modules`；对照 [1.10 章](../path/14-modules.md)  
- 回看 [1.9 FS](../path/13-fs19.md) 与 [1.8 shell](../path/12-shell18.md)

## 2. 官方下一站

| 系列 | 目标 | 节奏提示 |
|---|---|---|
| **1.10.y** | 更多模块、unload、registry 打磨 | 可停一阵 |
| **1.11** | 实用运行时（工具、I/O…） | backlog 选做 |
| **1.12** | Framebuffer（**延后**） | 实用主题够用后再开 |
| **2.0** | 平台集成发布 | 不赶 |

## 3. 自己玩

- 再写一个 badge 不同的小服务器，用 `modload PATH 0x….` 加载  
- 给 registry 加简单名字查询  

心态：DeepRoot 是**自研能力微内核**；学别人可以，不要搬 Linux kmod。
