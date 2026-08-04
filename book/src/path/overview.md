# 学习路线图（详细版）

DeepRoot 用类似 Linux 的版本号：

```text
MAJOR.PATCHLEVEL.SUBLEVEL
```

| 段 | 含义 | 初学者怎么用 |
|---|---|---|
| MAJOR | ABI 大断裂（0→1 表示稳定基线） | 文档基线在 **1.x** |
| PATCHLEVEL | **一类用户可见能力** | 按 0.1 → … → 1.4 读文档 |
| SUBLEVEL | 系列内修修补补 | 学概念时可忽略小版本差异 |

完整注释清单在仓库根目录 [`VERSION`](https://github.com/wuyijing-dev/DeepRoot/blob/main/VERSION)。那是路线图原文；本章是人话导航。

## 1. 总览图（到 1.4）

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
1.4  块设备教学替身 → 1.4.1 DRFS（路径可读块上文本）
```

## 2. 每段你「应该看见」什么

读完一章，用这条清单自测（比背定义有用）：

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
| 1.4 | 启动有 `block: ramdisk ready`…`DRFS`；`cat block.txt` 有内容 |

## 3. 推荐学习节奏

1. **第 0 天**：只做 [第一次启动](../intro/first-boot.md)，不改代码。  
2. **然后**：严格按下一章顺序；每章至少做「动手验证」里的一半。  
3. **卡壳时**：先查 [常见问题](../hands-on/faq.md)，再对照该章「易错点」。  
4. **想创造时**：完成 1.3 后再做 [自己写用户程序](../hands-on/write-user-prog.md)。

不要跳着改版本号学：请固定 `v1.4.1`（或用页首选择器打开冻结快照），否则日志和截图对不上。

## 4. 一句话心态

先求「跑起来、看见现象」，再求「为什么这样设计」。  
操作系统很大；DeepRoot 故意把教学关在 1.4 以内，就是为了让你能**走完一条路**。

下一章：[0.1 启动与串口](01-boot.md)。
