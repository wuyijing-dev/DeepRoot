# 下一步可以看什么

你已经走完 DeepRoot **1.12.0**（当前推荐 **`v1.12.0`**）：持久 DRFS、**fd 表**、sleep/ledger/hexdump。  

按根目录 `VERSION`：下一主题多为 **1.13** 服务命名；显示栈仍很远。

## 1. 巩固

- `hexdump note.txt` / `stat` / `sleep` / `ledger` / `caps`；对照 [1.11–1.12 章](../path/16-lab12.md)  
- 回看 [1.11 持久化](../path/15-fs11.md)

## 2. 官方下一站（摘要）

| 系列 | 目标 | 节奏提示 |
|---|---|---|
| **1.12.y** | virtio-console / 工具打磨 | 可选 |
| **1.13** | 服务命名 / 发现 | 下一主题 |
| **1.14–1.20** | grant → FB → 合成器 → 显示协议 | 长线 |
| **2.0–3.0** | 平台集成 → Wayland 启发里程碑 | 不赶 |

## 3. 自己玩

- 用 `open`/`fd_write` 思路自己写小工具（复制 hexdump）  
- 给 fd 加 O_APPEND 小实验  

心态：DeepRoot 是**自研能力微内核**。
