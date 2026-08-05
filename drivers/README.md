# drivers/ — userspace device drivers

DeepRoot is a **capability microkernel**: drivers are loadable **userspace**
ELF servers (not Linux `.ko` in the kernel address space).

Layout mirrors Linux’s *role* split (`drivers/` vs apps), not its in-kernel
implementation:

| Directory | Role |
|---|---|
| `drivers/` | Device protocol / hardware servers (`virtioblk`, `fbdemo`, `fbmenu`, …) |
| `user/` | Apps and canopy tasks (`init`, `shell`, `hello`, demos) |
| `kernel/` | Thin primitives only (sched, mm, caps, IPC) |

Each crate embeds via `kernel/build.rs` into ramfs (e.g. `/virtioblk`).
