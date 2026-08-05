# 下一步可以看什么

你已经走完 DeepRoot **1.15.2**（当前推荐 **`v1.15.2`**）：framebuffer 映射已落地；**后续路线以基础设施为主**，不再规划 UI / 合成器 / Wayland。

## 巩固

- [1.15 章](../path/20-fb15.md)：`fbdemo: fill_rect ok`（可选 HW）
- [剥离章](../path/19-peel.md)：`virtioblk` / Frame
- 根目录 [`VERSION`](https://github.com/wuyijing-dev/DeepRoot/blob/main/VERSION)：完整路线图

## 下一站（对标 Linux 角色，非桌面）

| 系列 | 目标 |
|---|---|
| **1.16** | 用户态 IRQ wait（irq 角色） |
| **1.17** | 存储继续 peel / 用户态 FS |
| **1.18** | Console / TTY 服务器 |
| **1.19** | virtio-net + 报文 I/O |
| **1.20** | mmap / 缺页 |
| **2.0** | 平台集成 |
