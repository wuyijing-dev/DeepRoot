# syscall 与 shell 内建

对齐：**v1.9.0**。

## 1. 新增号码（additive）

| 号 | 名字 | 作用 |
|---|---|---|
| 23 | `SYS_FS_MKDIR` | 建目录 |
| 24 | `SYS_FS_RMDIR` | 删空目录 |
| 25 | `SYS_FS_UNLINK` | 删 VFS 文件 |
| 26 | `SYS_CHDIR` | 设任务 cwd |
| 27 | `SYS_GETCWD` | 写出绝对路径 |

`SYS_FS_LIST` / `CAT` / `WRITE` 改为相对 **任务 cwd**（`a1=0` 的 list 列当前目录）。

## 2. 每任务 cwd

`UserTask.cwd_node` 默认根；`spawn`/`exec` 子任务继承父任务 cwd。  
shell 的 `cd` / `pwd` 走上述 syscall，不再只维护用户态前缀。

## 3. shell 内建

`mkdir` / `rmdir` / `rm` / `cd` / `pwd` / `ls [DIR]`。

## 4. 动手

```text
mkdir a
mkdir a/b
cd a/b
pwd
echo x > f.txt
cd ../..
ls a/b
rm a/b/f.txt
rmdir a/b
rmdir a
```
