# 1.5–1.6 设备树与 virtio-blk（对齐 v1.6.1）

到 **1.4** 你已经有：shell、ramfs、`DRFS` 文本、块层替身。  
从 **1.5 / 1.6** 起，板级描述与真块设备接上——本章对齐 **`v1.6.1`** 的设备树/virtio 内容；整站当前推荐标签是 **`v1.7.0`**（读完本章请继续 [1.7 SMP](11-smp.md)）。

## 本章拆读顺序

1. [自有设备树 `deeproot.dts`](fdt-virtio/01-own-dts.md)  
2. [跟读 `fdt.rs` 遍历器](fdt-virtio/02-fdt-walker.md)  
3. [virtio-blk 与 DRFS 后端](fdt-virtio/03-virtio-blk.md)  

## 1. 这一章要解决什么？

| 误解 | 纠正 |
|---|---|
| 「设备树是 Linux 专属」 | FDT 是通用固件约定；DeepRoot 用它描述教学板 |
| 「解析 QEMU 自动生成的 DTB 就够了」 | **1.6.1** 起仓库里有 **自己的** `.dts`，编译后用 `-dtb` 交给 OpenSBI |
| 「virtio 必须先上 PLIC 中断」 | 教学驱动用 **轮询** used 环即可（`virtio_blk.rs`） |

## 2. 一张图串起来

```text
platform/qemu-virt/deeproot.dts
        │  scripts/build-dtb.sh (dtc)
        ▼
build/deeproot-qemu-virt.dtb
        │  run-qemu.sh: -dtb …
        ▼
OpenSBI ──a1──► kernel_main ──► fdt::probe
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
                 memory         uart /          virtio,mmio[]
                 (mm)           MMIO map        virtio_blk::init
                                                   │
                                                   ▼
                                            block:: DRFS
                                                   │
                                                   ▼
                                            shell ls / cat
```

## 3. 版本对照（读日志时用）

| 标签 | 你该看见 |
|---|---|
| 1.4.x | `block: ramdisk ready` … `DRFS` |
| 1.6.0 | `fdt:` 发现行 + `virtio-blk: ready` + `block: virtio-blk ready` |
| **1.6.1** | 另有 `fdt: model "DeepRoot QEMU virt"` / `fdt: board deeproot,qemu-virt` |

## 4. 动手验收（请逐条做）

```bash
git checkout v1.6.1   # 或当前 main，以 VERSION 为准
./scripts/build-dtb.sh
./scripts/run-qemu.sh --smoke
```

进交互后再试：

```text
deeproot> ls
deeproot> cat block.txt
deeproot> cat blk-version
```

`ls` 应同时有 `fs: ramfs /` 与 `fs: block /`；`block.txt` 来自 **盘上 DRFS**（经 virtio）。

## 5. 和 1.4 章怎么衔接？

- [1.4 块设备](09-block.md) 仍讲 **DRFS 布局** 与路径 API。  
- 本章讲 **板级真源（DTS）** 与 **virtio 后端**。  
- [1.4.3](block/03-next-step-virtio.md) 已改成「指向本章」，不再假装 virtio 是遥远未来。

下一页：[自有设备树](fdt-virtio/01-own-dts.md)。
