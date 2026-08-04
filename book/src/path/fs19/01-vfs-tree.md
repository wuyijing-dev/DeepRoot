# VFS 树与路径

对齐：**v1.9.0**。

## 1. 根结点

`vfs::init` 占用结点 `0` 作为 `/`。最多约 48 个结点；名字最长 28、文件内容最长 256 字节（教学上限）。

## 2. 路径规则

- 以 `/` 开头 → 从根走
- 否则相对 **当前任务的 `cwd_node`**
- 支持 `.` / `..`（不能越过根）

`fs.rs` 在列出 `/` 时还会叠上 embed 与 DRFS 根条目，所以 `ls` 看起来像「一个」根目录。

## 3. 和 DRFS / embed 的边界

| 操作 | 行为 |
|---|---|
| `mkdir` / 嵌套 `>` 写 | 走 VFS |
| `cat hello` / `run hello` | embed（根级） |
| `cat block.txt` | DRFS（根级） |
| `rm version` | 拒绝（保护 embed） |

持久化目录与更大块写入留给后续 **1.9.y**。

## 4. 跟读

- `kernel/src/vfs.rs` — 树与 walk
- `kernel/src/fs.rs` — facade
