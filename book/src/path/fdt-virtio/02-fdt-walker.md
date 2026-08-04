# 跟读 `kernel/src/fdt.rs`

这一页只讲：**内核如何把 DTB 变成可用的 `Platform` 结构。**

对齐：**v1.6.1**（walker 自 1.5/1.6.0 起；model/board 日志在 1.6.1 强化）。

## 1. 它做什么 / 不做什么

| 做 | 不做 |
|---|---|
| 校验 magic `0xd00dfeed` | 不执行 DTS 文本（只认二进制 FDT） |
| 走 struct + strings，读 `compatible` / `reg` / `interrupts` | 不实现完整 Linux binding 动物园 |
| 填 `Platform`：memory、uart、virtio[]、framebuffer 提示 | 不直接驱动 UART / virtio |

驱动方再读 `fdt::get()`：`mm` 用 memory，`virtio_blk` 扫 `virtio,mmio`。

## 2. 启动顺序里谁先谁后

[`kernel_main`](../boot/03-kernel-main.md) 大致：

```text
trap::init
fdt::probe(dtb_pa)     ← 此时尚未开 Sv39；DTB 在 DRAM，可按物理地址读
mm::init               ← memmap::discover → fdt::memory_reg()
map_mmio_range(…)      ← UART / virtio / 可选 FB
block::init            ← 内部 virtio_blk::init()
timer / servers
```

## 3. `Platform` 里有什么（心智模型）

```text
Platform {
  model / board_compat     // 根节点（1.6.1 日志）
  memory: Option<Reg>      // /memory@…
  uart: Option<UartDev>    // ns16550a 等
  virtio[0..N]             // compatible = "virtio,mmio"
  framebuffer: Option<…>   // 像素显示约 1.15；FDT 仍可先记下 hint
}
```

`Reg { base, size }` 来自 `reg` 属性，并尊重父节点的 `#address-cells` / `#size-cells`。

## 4. 和早期 `memmap` 的关系

0.2 时代：`memmap.rs` 里内嵌了一个「只找 memory」的迷你 walker。  
1.5 起：共享逻辑在 **`fdt.rs`**；`memmap::discover` 只消费 `fdt::memory_reg()`，失败再回退常量。

读内存章时：[0.2.1 RAM 发现](../mm/01-memory-map.md) —— 概念仍对，实现入口已换成 FDT 模块。

## 5. 你该观察什么

- `fdt: blob pa=…`：证明 `a1` 有效  
- `fdt: model` / `fdt: board`：证明是 **我们的** 树（1.6.1）  
- `fdt: virtio-mmio count=8`：与 DTS 里八个槽一致  
- 若 `memory` 离谱：先查 DTS 里 `memory@80000000` 的 `reg` 是否与 `-m` 一致  

## 6. 易错点

| 现象 | 常见原因 |
|---|---|
| `fdt: no DTB` | `a1=0`；或未传 `-dtb` 且固件也没给 |
| 有 blob 无 virtio | DTS 缺 `virtio,mmio`，或 `compatible` 拼错 |
| 开页表后读 MMIO 炸 | 忘记 `map_mmio_range`（DRAM 身份映射盖不住 `0x1000_xxxx`） |

下一页：[virtio-blk 与 DRFS](03-virtio-blk.md)。
