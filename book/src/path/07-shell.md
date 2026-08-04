# 1.2 Shell（详细跟读）

shell 是你最常触摸的用户程序。把它读懂，你就打通了「串口字节 → 解析 → syscall」整条链。

## 本章拆读顺序

1. [1.2.1 shell 主循环](shell/01-main-loop.md)
2. [1.2.2 `read_line` 与共享串口](shell/02-read-line.md)
3. [1.2.3 `run_path` 与前台等待](shell/03-run-path.md)

## 1. 主循环结构（`user/shell/src/main.rs`）

伪代码：

```text
打印 ready 行
loop:
  打印 "deeproot> "
  n = read_line(buf)
  line = trim(buf[..n])
  若空行: continue
  匹配 help / ls / cat / run / hello / badapple / exit
  否则: unknown
```

### 1.1 `read_line` 做什么？

循环：

1. `sys::debug_read_byte()`  
   - `>=0`：得到字节  
   - `<0`（通常 `-11`）：暂时无输入 → `yield` 再试  
2. `\r` / `\n`：回显换行并结束一行  
3. `0x7f` / `0x08`：退格，回显 `\b \b`  
4. 其它控制字符：丢弃  
5. 可打印字符：写入 buf 并回显  

这就是为什么「内核狂打日志」会让你感觉输不了字：串口是共享的，而且调度在不停切换。

### 1.2 `run_path`

```text
id = sys::exec(path)
失败则打印 exec failed
否则 loop wait(id) until not ERR_AGAIN
```

所以 `run hello` 期间提示符不会回来，直到 hello 退出——这是刻意的前台运行模型。

## 2. 内核侧读串口

`SYS_DEBUG_READ` → `sbi::console_getchar`：

- Legacy EID 必须是 **`0x02`**  
- 返回值在 `a0`（legacy），封装成 `Option<u8>`  
- 没有字符 → `None` → 用户态看到负数  

写串口：`SYS_DEBUG_WRITE` 把用户缓冲区拷到内核再 `console_write`（批量 DBCN，失败则逐字节）。

## 3. 动手验证（请做）

1. 输入 `he` 再退格再改成 `help`，确认退格体验。  
2. 只按回车：应安静地再出提示符。  
3. 输入 `help` 以外的词：应 `unknown`。  
4. `run hello` 时观察：是否必须等 hello 结束后才出现下一行提示符。

## 4. 自己加一个 builtin（练习）

例如加 `yield` 命令：解析到后调用 `sys::yield_now()` 几次。  
这能帮你确认：「shell 改动 → 重编内核嵌入 → QEMU 可见」。

## 5. 易错点

| 现象 | 原因 |
|---|---|
| 每个命令都 unknown 且无回显 | getchar EID/返回值解析错（经典坑） |
| 有回显但命令总不对 | trim / `\r` 没剥干净 |
| run 后提示符消失很久 | 子任务没 exit，或 wait 没用上 |

下一章：[1.3 ramfs 与 run](08-fs.md)。
