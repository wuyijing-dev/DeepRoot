# 1.4.3 以后怎么走向 virtio-blk

这一页只讲：**如果以后要把 stand-in 换成真块设备，应该怎么想。**

## 1. 理想分层

```text
更高层文件系统 / 路径接口
    ↓
block 抽象（read/write）
    ↓
具体后端：ramdisk 或 virtio-blk
```

## 2. 为什么现在先不展开

因为这已经不属于 1.4.0 主线教学的必要难度了。  
你先把“路径 -> exec -> 用户任务”和“block stand-in 的存在理由”读懂，收获更大。

## 3. 真正继续做下去时要补什么

- QEMU 挂真实磁盘镜像
- virtio-blk 设备发现
- descriptor/queue
- 同步与中断
- 更高层文件系统如何消费 block 接口

下一节建议去：[自己写一个用户程序](../../hands-on/write-user-prog.md)

