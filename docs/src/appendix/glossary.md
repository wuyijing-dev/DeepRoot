# 名词表

按字母 / 拼音混排不重要；用搜索（站点自带 search）更快。

| 词 | 一句话 |
|---|---|
| **ABI** | 用户程序与内核之间的二进制约定（syscall 号、结构体布局）。见 `deeproot-abi`。 |
| **AddrSpace** | 一个任务的虚拟地址空间（自己的页表根）。 |
| **badge** | 能力或端点上的标记，用来区分「同一类服务的不同身份」。 |
| **BSS** | 未初始化全局/静态数据段；开机必须清零。 |
| **capability（能力）** | 受内核管理的「权限票」，持有者才能对某对象执行某些操作。 |
| **cap / CSpace** | 能力与能力表（任务持有哪些票）。 |
| **DBCN** | SBI Debug Console 扩展：批量写控制台。 |
| **DTB** | Device Tree Blob，固件传给内核的硬件描述（DeepRoot 早期可能只用一部分）。 |
| **ecall** | RISC-V 环境调用指令；用户态进内核做 syscall。 |
| **ELF** | 可执行文件格式；用户程序以 ELF 嵌入并加载。 |
| **Endpoint** | IPC 通信端点对象；常通过能力引用。 |
| **hart** | Hardware thread，一颗硬件硬件线程/核。 |
| **idle** | 无可运行任务时睡的内核任务（常 `wfi`）。 |
| **IPC** | 进程/任务间通信；DeepRoot 教学树以同步 call/recv/reply 为主。 |
| **Ledger** | Root Ledger，环形事件账本，便于观察启动与 IPC。 |
| **M / S / U** | RISC-V 特权级：Machine / Supervisor / User。 |
| **mdBook** | 本学习站点的生成器。 |
| **OpenSBI** | 常见的 M-mode 固件实现；提供 SBI 调用。 |
| **page fault** | 访问未映射/权限不足的虚址时触发的陷阱。 |
| **preempt** | 抢占：时钟中断打断当前任务，调度另一个。 |
| **provenance** | 能力来源：这张票是怎么 mint/derive 出来的。 |
| **QEMU virt** | 常用的虚拟开发板机型；DeepRoot 脚本默认用它。 |
| **ramfs** | 教学内存文件系统：名字→静态字节。 |
| **ramdisk** | 用内存假装磁盘；1.4 块层替身。 |
| **SBI** | Supervisor Binary Interface：S-mode 向固件借的服务。 |
| **satp** | 控制页表根与模式的 CSR；切任务时常切 satp。 |
| **sched id** | 调度器任务槽编号；`SYS_EXEC`/`SYS_WAIT` 使用。 |
| **sepc** | 陷阱发生时的 PC；从用户态返回时用它恢复。 |
| **shell** | DeepRoot 用户态交互程序，不是 bash。 |
| **sret** | 从陷阱返回的指令。 |
| **stvec** | 陷阱入口地址寄存器。 |
| **Sv39** | RISC-V 39 位虚址页表模式（三级页表）。 |
| **syscall** | 系统调用；DeepRoot 用 `a7` 放号码。 |
| **TCB** | 任务控制块：调度器眼里的任务状态。 |
| **trap frame** | 保存/恢复用户寄存器的结构。 |
| **virtio-blk** | 虚拟化块设备标准；1.4 用 ramdisk 替身占位。 |
| **wfi** | Wait For Interrupt，闲时省电/等待。 |
| **Zombie** | 已退出、等待被 `wait` 回收的任务状态。 |
