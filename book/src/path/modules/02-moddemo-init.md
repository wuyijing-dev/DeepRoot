# moddemo / modnote 与 init

对齐：**v1.10.1**。

## moddemo

- 路径：`/moddemo`（embed；`ls` 可见）
- badge：`0xD001`
- 日志：`moddemo: online` / `moddemo: pong`

## modnote（1.10.1）

- 路径：`/modnote`（embed）以及可 `cp` 到 VFS 名如 `mynote`
- badge：`0xD002`（默认；也可 `modload PATH badge`）
- 日志：`modnote: online` / `modnote: noted`

**都不**在 `servers::bring_up` 里启动——满足「先无模块 canopy，再加载」。

## init 流程

1. 原有 ping / console / hello  
2. `spawn_server("moddemo", 0xD001)` → `init: module loaded`  
3. `ipc_call` → `init: module call ok`  
4. `fs_cp("modnote", "mynote")` → `init: cp modnote -> mynote ok`  
5. `spawn_server("mynote", 0xD002)` → `init: vfs module loaded` / `call ok`  
6. `handing off to shell`

## shell

```text
cp modnote othernote
modload othernote 0xd003
modules
```
