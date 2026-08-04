# Summary

[前言](README.md)

---

# 入门

- [这是什么？](intro/what-is-deeproot.md)
- [你需要准备什么](intro/prerequisites.md)
- [第一次启动](intro/first-boot.md)
- [仓库长什么样](intro/repo-map.md)

# 跟着版本学

- [学习路线图](path/overview.md)
- [0.1 启动与串口](path/01-boot.md)
  - [0.1.1 从 QEMU 到 `_start`](path/boot/01-boot-path.md)
  - [0.1.2 跟读 `boot.rs`](path/boot/02-boot-rs.md)
  - [0.1.3 跟读 `kernel_main`](path/boot/03-kernel-main.md)
  - [0.1.4 SBI 控制台](path/boot/04-sbi-console.md)
  - [0.1.5 early trap 与 `stvec`](path/boot/05-early-trap.md)
- [0.2 内存与页表](path/02-mm.md)
  - [0.2.1 RAM 发现与 DTB 回退](path/mm/01-memory-map.md)
  - [0.2.2 frame allocator 与 heap](path/mm/02-frame-heap.md)
  - [0.2.3 Sv39 身份映射](path/mm/03-sv39.md)
  - [0.2.4 为什么这决定 ELF 装载](path/mm/04-elf-preview.md)
- [0.3–0.4 能力与 IPC](path/03-cap-ipc.md)
  - [0.3.1 Capability 模型](path/cap-ipc/01-cap-model.md)
  - [0.3.2 启动时的 CSpace 安装](path/cap-ipc/02-boot-cspace.md)
  - [0.4.1 call / recv / reply](path/cap-ipc/03-ipc-call-flow.md)
  - [0.4.2 Root Ledger 怎么看](path/cap-ipc/04-ledger.md)
  - [0.4.3 IPC 与调度状态切换](path/cap-ipc/05-ipc-sched.md)
- [0.5–0.6 用户态与调度](path/04-user-sched.md)
  - [0.5.1 用户程序最小骨架](path/user-sched/01-user-runtime.md)
  - [0.5.2 `servers::bring_up` 跟读](path/user-sched/02-bring-up.md)
  - [0.6.1 TaskState 与 BlockReason](path/user-sched/03-task-states.md)
  - [0.6.2 timer / preempt](path/user-sched/04-timer-preempt.md)
  - [0.6.3 syscall 返回值到底写给谁](path/user-sched/05-syscall-return.md)
- [1.0 冻结 ABI](path/05-abi.md)
  - [1.0.1 寄存器调用约定](path/abi/01-registers.md)
  - [1.0.2 错误码与核心 syscall](path/abi/02-errors-core-syscalls.md)
  - [1.0.3 `trap.rs` 如何解码 ecall](path/abi/03-trap-decode.md)
  - [1.0.4 四个 syscall 实战跟读](path/abi/04-guided-syscalls.md)
- [1.1 地址空间与 spawn](path/06-as-spawn.md)
  - [1.1.1 per-task 页表](path/as-spawn/01-per-task-as.md)
  - [1.1.2 `SYS_SPAWN` 控制流](path/as-spawn/02-sys-spawn.md)
  - [1.1.3 跟读 `elf.rs`](path/as-spawn/03-elf-loader.md)
  - [1.1.4 Zombie 与 `SYS_WAIT`](path/as-spawn/04-zombie-wait.md)
- [1.2 Shell](path/07-shell.md)
  - [1.2.1 shell 主循环](path/shell/01-main-loop.md)
  - [1.2.2 `read_line` 与共享串口](path/shell/02-read-line.md)
  - [1.2.3 `run_path` 与前台等待](path/shell/03-run-path.md)
- [1.3 ramfs 与 run](path/08-fs.md)
  - [1.3.1 ramfs 模型](path/fs/01-ramfs-model.md)
  - [1.3.2 `build.rs` 如何产出嵌入字节](path/fs/02-build-pipeline.md)
  - [1.3.3 `FS_LIST` / `FS_CAT` / `EXEC`](path/fs/03-fs-syscalls.md)
  - [1.3.4 `SYS_SPAWN` vs `SYS_EXEC`](path/fs/04-spawn-vs-exec.md)
- [1.4 块设备（教学替身）](path/09-block.md)
  - [1.4.1 为什么先做替身](path/block/01-why-standin.md)
  - [1.4.2 跟读 `block.rs`](path/block/02-read-block-rs.md)
  - [1.4.3 走向 virtio（见 1.5–1.6）](path/block/03-next-step-virtio.md)
- [1.5–1.6 设备树与 virtio-blk](path/10-fdt-virtio.md)
  - [1.6.1 自有设备树 `deeproot.dts`](path/fdt-virtio/01-own-dts.md)
  - [1.5 跟读 `fdt.rs`](path/fdt-virtio/02-fdt-walker.md)
  - [1.6 virtio-blk 与 DRFS 后端](path/fdt-virtio/03-virtio-blk.md)

# 动手玩

- [Shell 常用命令](hands-on/shell-commands.md)
- [自己写一个用户程序](hands-on/write-user-prog.md)
  - [步骤 1：复制 hello 模板](hands-on/write-user-prog/01-clone-template.md)
  - [步骤 2：注册到 workspace 与 build](hands-on/write-user-prog/02-register-build.md)
  - [步骤 3：挂进 ramfs 与 shell](hands-on/write-user-prog/03-ramfs-shell.md)
  - [步骤 4：构建、运行、调试](hands-on/write-user-prog/04-build-debug.md)
- [常见问题](hands-on/faq.md)

# 附录

- [名词表](appendix/glossary.md)
- [版本与标签](appendix/versions.md)
- [下一步可以看什么](appendix/next.md)
