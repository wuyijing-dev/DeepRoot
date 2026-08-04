# 1.10 可加载模块 / 动态服务器（对齐 v1.10.0）

对齐：**v1.10.0**。

当前推荐标签：**`v1.10.0`**。

## 这一站解决什么？

启动时的 canopy（ping / console / init / shell）仍是 **build-time embed**。  
**1.10.0** 增加：启动之后再从路径拉起一个可选用户态服务器，并给调用者发一张 endpoint 能力。

- 演示 ELF：`/moddemo`（**不**进 `bring_up`）
- syscall：`SYS_SPAWN_SERVER`（28）、`SYS_MODULE_LIST`（29）
- init 在交接 shell 前会 `spawn_server("moddemo")` 并 `ipc_call`
- shell：`modload` / `modules`

## 跟读

| 文件 | 作用 |
|---|---|
| `kernel/src/module.rs` | 简易 registry |
| `kernel/src/sched.rs` | `SYS_SPAWN_SERVER` |
| `user/moddemo/` | 可选 IPC 服务器 |
| `user/init/` | 启动后加载并调用 |

## 动手

```text
modules
modload moddemo
```

（init 已加载过一次时，再次 `modload` 可能因 badge 冲突失败——属预期；换 badge 或后续 1.10.y 做 unload。）

## 验收

```bash
git checkout v1.10.0
./scripts/run-qemu.sh --smoke
```

应出现 `module: loaded 'moddemo'`、`moddemo: pong`、`init: module call ok`。

## 子页

- [SYS_SPAWN_SERVER](modules/01-spawn-server.md)
- [moddemo 与 init](modules/02-moddemo-init.md)
