# 1.3.3 `FS_LIST` / `FS_CAT` / `EXEC`

这一页只讲：**shell 的 `ls` / `cat` / `run` 在内核里各自走哪条路径。**

## 1. `FS_LIST`

`ls` -> `SYS_FS_LIST` -> `fs::list()`  
作用是把当前静态表直接打印出来。

## 2. `FS_CAT`

`cat path` -> `SYS_FS_CAT`

内核会：

1. 读 path 指针和长度
2. 查 ramfs
3. 如果是文本就打印
4. 如果是 ELF 就提示你用 `run`

## 3. `EXEC`

`run path` -> `SYS_EXEC`

内核会：

1. 找到该路径对应的字节
2. 校验是不是 ELF
3. 走 `spawn_elf_bytes(...)`
4. 返回 child sched id

## 4. 为什么列表输出是“内核日志风”

因为当前教学树优先的是“你能直接看见”，而不是先设计一套复杂的目录项缓冲区 ABI。

下一页：[1.3.4 `SYS_SPAWN` vs `SYS_EXEC`](04-spawn-vs-exec.md)

