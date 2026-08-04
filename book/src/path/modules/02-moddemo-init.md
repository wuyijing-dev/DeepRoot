# moddemo 与 init

对齐：**v1.10.0**。

## moddemo

- 路径：`/moddemo`（embed；`ls` 可见）
- badge：`0xD001`
- 日志：`moddemo: online` / `moddemo: pong`

**不**在 `servers::bring_up` 里启动——满足「先无模块 canopy，再加载」。

## init 流程

1. 原有 ping / console / hello  
2. `spawn_server("moddemo", 0xD001)` → `init: module loaded`  
3. `ipc_call(slot, …)` → `init: module call ok`  
4. `handing off to shell`

## shell

```text
modload moddemo
modules
```
