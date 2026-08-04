# DeepRoot platform trees (device tree sources)

DeepRoot keeps **its own** DTS in-tree. QEMU still provides the virtual
hardware; this tree is how we *name and pin* that board for teaching.

| Path | Role |
|------|------|
| [`qemu-virt/deeproot.dts`](qemu-virt/deeproot.dts) | Source of truth for the QEMU `virt` teaching board |
| `../build/deeproot-qemu-virt.dtb` | Built blob (`scripts/build-dtb.sh`) |
| `../scripts/run-qemu.sh` | Passes `-dtb` so OpenSBI hands **our** blob to the kernel (`a1`) |

```bash
./scripts/build-dtb.sh
./scripts/run-qemu.sh --smoke   # greps fdt model / deeproot,qemu-virt
```

Kernel walker: `kernel/src/fdt.rs` (does not invent nodes; it reads this blob).
