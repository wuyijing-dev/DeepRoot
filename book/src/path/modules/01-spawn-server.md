# SYS_SPAWN_SERVER

对齐：**v1.10.1**。

## 调用约定

| 寄存器 | 含义 |
|---|---|
| a7 | 28 |
| a0/a1 | 路径指针与长度（embed 根名，如 `moddemo`） |
| a2 | badge（须唯一；moddemo 默认 `0xD001`） |
| 返回 | 调用者 CapSpace 里新 endpoint 的 **slot**；失败为负 |

内核步骤：`fs::lookup` → `tasks.spawn` → `eps.create` → `spawn_elf_bytes` → `install_copy` → `module::register`。

## 与 SYS_EXEC 的差别

| | `SYS_EXEC` | `SYS_SPAWN_SERVER` |
|---|---|---|
| 用途 | 跑完就退出的程序 | 常驻 IPC 服务器 |
| 能力 | 子任务空 CSpace | 给**调用者**发 EP |
| 注册表 | 无 | `module` registry |

`SYS_MODULE_LIST`（29）只打印 registry。
