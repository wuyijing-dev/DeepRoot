# 下一步可以看什么

你已经走完 DeepRoot **1.14.2**（当前推荐 **`v1.14.2`**）：grant + 剥离起步（MMIO / virtioblk probe）。  

下一主题仍是 **1.15** framebuffer；同时可继续剥块驱动 / FS 策略到 userspace。

## 巩固

- [1.14 章](../path/18-grant14.md)：`grantpeer: saw magic` → `frame revoke ok`
- [剥离章](../path/19-peel.md)：`virtioblk: probe ok`

## 下一站

| 系列 | 目标 |
|---|---|
| **1.14.y / peel** | IRQ caps、userspace 队列、再切 DRFS |
| **1.15** | Framebuffer 像素 / 简单 UI |
| **1.16+** | 输入 → 合成器 → 显示协议 |

心态：DeepRoot 是**自研能力微内核**；优先 userspace 模块，内核只留薄原语。
