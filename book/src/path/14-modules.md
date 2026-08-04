# 1.10 可加载模块 / 动态服务器（对齐 v1.10.1）

对齐：**v1.10.1**。

当前推荐标签：**`v1.10.1`**。

## 这一站解决什么？

启动时的 canopy（ping / console / init / shell）仍是 **build-time embed**。  
**1.10** 增加：启动之后再从路径拉起可选用户态服务器。

- 演示 ELF：`/moddemo`、`/modnote`（**不**进 `bring_up`）
- **1.10.0**：`SYS_SPAWN_SERVER` + embed 名 `moddemo`
- **1.10.1**：`SYS_FS_CP` / shell `cp`；`FILE_MAX` 够装小 ELF；`modload` **从 VFS 文件**加载
- syscall：`SYS_SPAWN_SERVER`（28）、`SYS_MODULE_LIST`（29）、`SYS_FS_CP`（30）

## 动手

```text
modules
# embed 名（init 已加载过 badge 0xD001 的 moddemo，再加载会冲突）：
modnote
# 或从 VFS 文件加载：
cp modnote othernote
modload othernote 0xd003
modules
```

## 验收

```bash
git checkout v1.10.1
./scripts/run-qemu.sh --smoke
```

应出现 `module: loaded 'moddemo'`、`init: cp modnote -> mynote ok`、`module: loaded 'mynote'`、`init: vfs module call ok`。

## 子页

- [SYS_SPAWN_SERVER](modules/01-spawn-server.md)
- [moddemo 与 init](modules/02-moddemo-init.md)
