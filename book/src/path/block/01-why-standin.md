# 为什么先做替身

这一页只讲：**为什么 1.4 不是直接上完整 virtio-blk。**

## 1. 教学权衡

如果在新手刚学完 shell / ramfs / exec 之后，马上丢给他完整 virtio 队列与持久化 FS，曲线会陡到很难继续。

## 2. 替身的价值

ramdisk stand-in 提供的是：

- 启动路径里真的有 block 模块  
- `read` / `write` 这类接口形状  
- 从 1.4.1 起还有 DRFS 布局，让 `ls` / `cat` 真正读到「盘上」的文本  

真实 virtio 可以以后再换 backend，不必先改 shell 习惯。

下一页：[跟读 `block.rs`](02-read-block-rs.md)
