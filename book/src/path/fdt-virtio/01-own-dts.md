# 自有设备树：`platform/qemu-virt/deeproot.dts`

这一页只讲：**DeepRoot 自己的板级描述写在哪、怎么进到内核手里。**

对齐标签：**v1.6.1**。

## 1. 为什么要「自己的」树？

| 做法 | 结果 |
|---|---|
| 只用 QEMU 自动生成的 DTB | 内核能发现设备，但仓库里**没有**可读、可 diff 的板级契约 |
| 维护 `deeproot.dts` + `-dtb` | 板级地址、兼容串、model 都在 git 里；smoke 能核对 `deeproot,qemu-virt` |

硬件仍是 QEMU `virt`；我们**不发明**另一套 MMIO，而是把已知布局写成 **DeepRoot 署名的** DTS。

## 2. 源文件在哪？

```text
platform/
├── README.md
└── qemu-virt/
    └── deeproot.dts      ← 真源
scripts/build-dtb.sh      ← dtc → build/deeproot-qemu-virt.dtb
scripts/run-qemu.sh       ← -dtb build/deeproot-qemu-virt.dtb
```

打开 DTS，根节点大致是：

```dts
/ {
    model = "DeepRoot QEMU virt";
    compatible = "deeproot,qemu-virt", "riscv-virtio";
    ...
    memory@80000000 { ... };          /* 与 -m 256M 对齐 */
    soc {
        uart@10000000 { compatible = "ns16550a"; ... };
        virtio_mmio@10001000 { compatible = "virtio,mmio"; ... };
        /* … 共 8 个 virtio-mmio 槽 … */
        clint@2000000 { ... };
        interrupt-controller@c000000 { ... };  /* PLIC */
    };
};
```

## 3. 编译与启动链

```bash
./scripts/build-dtb.sh
# → build/deeproot-qemu-virt.dtb
```

`run-qemu.sh` 会先编 DTB，再：

```text
qemu-system-riscv64 … -kernel deeproot-kernel -dtb deeproot-qemu-virt.dtb …
```

OpenSBI 把该 blob 的物理地址放进 **`a1`**，`_start` 原样交给 `kernel_main(hartid, dtb_pa)`。

## 4. 你该在日志里看见什么

```text
fdt: blob pa=0x… size=… version=17
fdt: model "DeepRoot QEMU virt"
fdt: board deeproot,qemu-virt
fdt: memory 0x80000000..0x90000000 (256 MiB)
fdt: uart ns16550a @ 0x10000000 …
fdt: virtio-mmio count=8
```

若没有 `model` / `board` 行：多半仍在用自动 DTB（旧脚本未传 `-dtb`），或标签早于 **1.6.1**。

## 5. 依赖工具

需要主机上的 **`dtc`**（Debian/Ubuntu：`device-tree-compiler`）。  
见 [你需要准备什么](../../intro/prerequisites.md)。

## 6. 动手小实验

1. 改 `model` 字符串里加一个后缀，重跑 `build-dtb.sh` + QEMU，看日志是否变。做完请还原。  
2. `dtc -I dtb -O dts build/deeproot-qemu-virt.dtb | less` —— 确认编出来的内容和源接近。  
3. **不要**随便改 `reg` 地址：必须与 QEMU virt 实际 MMIO 一致，否则 virtio/OpenSBI 会挂。

下一页：[跟读 `fdt.rs`](02-fdt-walker.md)。
