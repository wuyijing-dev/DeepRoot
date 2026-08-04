# 1.4.1 为什么先做替身

这一页只讲：**为什么 1.4 不是直接上完整 virtio-blk。**

## 1. 教学权衡

如果在新手刚学完：

- shell
- ramfs
- exec

之后，马上丢给他完整 virtio 队列、descriptor、持久化 FS，学习曲线会陡到很难继续。

## 2. 替身的价值

ramdisk stand-in 提供的是：

- 启动路径里真的有 block 模块
- 真有 `read` 这类接口形状
- 将来能替换后端

但不会一口气把设备协议复杂度全倒给读者。

下一页：[1.4.2 跟读 `block.rs`](02-read-block-rs.md)

