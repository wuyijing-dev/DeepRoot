# 1.4.3 走向 virtio（见 1.5–1.6）

1.4 教学路径先交 **ramdisk + DRFS**。真设备与自有设备树已经在后续系列落地。

## 分层回顾

```text
更高层路径 API（ls / cat）
    ↓
block / DRFS
    ↓
后端：ramdisk（1.4）或 virtio-blk（1.6）
```

## 请转到新章

完整说明（对齐 **v1.6.1**）在：

- [1.5–1.6 设备树与 virtio-blk](../10-fdt-virtio.md)  
- [自有 `deeproot.dts`](../fdt-virtio/01-own-dts.md)  
- [virtio-blk 与 DRFS](../fdt-virtio/03-virtio-blk.md)  

不必在 1.4 里把 virtio 规范读完；把路径 API 弄清楚后，按上面三页跟读即可。
