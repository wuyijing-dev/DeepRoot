# 学习路线图（详细版）

DeepRoot 用类似 Linux 的版本号：

```text
MAJOR.PATCHLEVEL.SUBLEVEL
```

| 段 | 含义 | 初学者怎么用 |
|---|---|---|
| MAJOR | ABI 大断裂或平台跃迁（2.0 = 集成发布，不赶） | 文档基线在 **1.x**，当前推荐 **v1.12.0** |
| PATCHLEVEL | 一个主题系列（放慢；系列内多用 SUBLEVEL） | 按 0.1 → … → 1.10 读文档 |
| SUBLEVEL | 系列内分段落地与打磨 | 例：1.10.0 → 1.10.1 → … |

完整注释清单在仓库根目录 [`VERSION`](https://github.com/wuyijing-dev/DeepRoot/blob/main/VERSION)。

## 1. 总览图（到 1.8）

```text
0.1  启动、串口、trap、Ledger 雏形
0.2  物理内存、堆、Sv39 页表
0.3  Capability（能力票）
0.4  同步 IPC + Ledger 事件
0.5  用户态 ELF 服务器（init/console/ping）
0.6  调度、时钟抢占、idle
0.7–0.9  收敛、稳定、冻结准备
1.0  ABI 冻结基线（syscall 号码钉死）
1.1  每任务地址空间 + SYS_SPAWN
1.2  交互 shell（读串口、解析命令）
1.3  ramfs + 按路径 SYS_EXEC
1.4  块设备教学替身 → DRFS 文本
1.5  FDT 平台发现（fdt.rs）
1.6  virtio-blk + DRFS；1.6.1 自有 deeproot.dts
1.7  SMP：HSM 二级核、每 hart RQ、锁与 IPI
1.8  自研 shell：argv/env/history/&/|/\>
```

## 2. 每段你「应该看见」什么

| 系列 | 你应该能指着现象说 |
|---|---|
| 0.1 | 这行字来自内核 `println!` / SBI |
| 0.2 | 日志里有 `Sv39 identity map` |
| 0.3–0.4 | init 能和 ping/console 说话（IPC） |
| 0.5–0.6 | 多个用户程序轮流跑，时钟可抢占 |
| 1.0 | `deeproot-abi` 里的号码与内核一致 |
| 1.1 | `spawn`/`run` 出来的 hello 有自己的地址空间 |
| 1.2 | 你能在 `deeproot>` 打字 |
| 1.3 | `ls`/`cat`/`run` 对应 ramfs |
| 1.4 | `DRFS`；`cat block.txt` 有内容 |
| 1.5–1.6 | `fdt: board deeproot,qemu-virt`；`virtio-blk: ready` |
| **1.7** | `smp: 2 hart(s) online`；两条 `timer: hart=` / `idle hart=` |
| **1.8** | `shell: DeepRoot shell 1.8 ready`；`help` 含 \| / > / & |
| **1.9** | `vfs: in-RAM tree ready`；`mkdir` / 真实 `cd`；`shell … 1.9 ready` |
| **1.10** | `module: loaded 'moddemo'`；`moddemo: pong`；`shell … 1.10 ready` |

## 3. 推荐学习节奏

1. **第 0 天**：只做 [第一次启动](../intro/first-boot.md)，不改代码。  
2. **然后**：按章节顺序读到 **1.10**；每章至少做一半「动手验证」。  
3. **卡壳时**：先查 [常见问题](../hands-on/faq.md)。  
4. **想创造时**：完成 1.3 后再做 [自己写用户程序](../hands-on/write-user-prog.md)。

请固定 **`v1.12.0`**（或页首选择器打开对应冻结快照），否则日志和截图对不上。

## 4. 一句话心态

先求「跑起来、看见现象」，再求「为什么这样设计」。  
1.12 起有 sleep/ledger/hexdump；显示路径见 `VERSION` / [下一步](../appendix/next.md)。

下一章：[0.1 启动与串口](01-boot.md)。
