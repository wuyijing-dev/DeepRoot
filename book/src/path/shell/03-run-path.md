# `run_path` 与前台等待

这一页只讲：**`run hello` 为什么会「卡住」直到程序结束。**

## 1. 控制流

```text
id = SYS_EXEC(path)
if id < 0:
    打印 exec failed
else:
    loop:
        st = SYS_WAIT(id)
        if st == ERR_AGAIN: yield; continue
        break   # 子任务已退出
```

提示符在 `wait` 成功返回之前不会再打印——这就是前台模型。

## 2. EXEC 失败常见原因

- 路径不在 embed `FILES`（块上文本不能 `run`）  
- 字节不是 ELF  
- 地址空间 / 装载失败（少见，看内核日志）  

## 3. 和 `cat` 的分工

| 命令 | 目的 |
|---|---|
| `cat hello` | 发现是 ELF → 提示用 `run` |
| `run hello` | 真正创建用户任务 |

下一章：[1.3 ramfs 与 run](../08-fs.md)
