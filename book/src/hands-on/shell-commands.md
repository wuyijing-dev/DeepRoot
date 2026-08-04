# Shell 常用命令（详细手册）

提示符：`deeproot>`  
解析器在 `user/shell/src/main.rs`：非常朴素——**没有**引号、管道、变量、通配符。

## 1. 命令一览

| 你输入 | 实际行为 | 底层 |
|---|---|---|
| `help` | 打印内置帮助文本 | 仅用户态字符串 |
| `ls` | 列出 embed ramfs + 块上 DRFS | `SYS_FS_LIST` |
| `cat <文件>` | 显示文本；ELF 则提示 `run` | `SYS_FS_CAT` |
| `run <名字>` | 加载 ELF，前台等到退出 | `SYS_EXEC` + `SYS_WAIT` |
| `hello` | 等同 `run hello` | 同上 |
| `badapple` | 等同 `run badapple` | 同上（耗时长） |
| `exit` | shell 退出 | `SYS_EXIT` |
| （空行） | 再出提示符 | — |
| 其它 | `shell: unknown - type: help` | — |

## 2. 推荐练习顺序（请照做）

### 练习 A：确认输入链路

1. 输入 `help`，应看到多行帮助。  
2. 输入 `he`，退格删掉，改成 `help`。  
3. 只按回车：应安静地再出现 `deeproot>`。

若「一按键就 unknown」或完全无回显 → 见 [FAQ：不能输入](faq.md)。

### 练习 B：认文件系统

```text
deeproot> ls
deeproot> cat readme.txt
deeproot> cat version
deeproot> cat block.txt
deeproot> cat from-block
deeproot> cat hello
```

期望：

- `ls` 先有 `fs: ramfs /`，再有 `fs: block /`（含 `block.txt` 等）  
- `cat version` → `1.4.1`（embed）  
- `cat block.txt` → 一段说明文字（DRFS / 块上）  
- 对 `hello` 提示用 `run`，不要刷二进制  

### 练习 C：前台运行

```text
deeproot> run hello
```

期望顺序：

1. `hello: spawned ELF says hi`  
2. 再次出现 `deeproot>`  

在 hello 结束前不应插入新的提示符（其它任务的日志仍可能插进来——单串口共享）。

### 练习 D：（可选）彩蛋

```text
deeproot> badapple
```

- 播放中可按 `q` 退出  
- 结束行关注 `drawn=` 是否接近总帧；若很少，可能是解码中途失败  

Bad Apple **不是**教学主线，跳过完全没问题。

## 3. 行为细节（避免踩坑）

### 3.0 输入回显与退格（来自 `read_line`）

shell 的输入并不是“读取一整行再解析”，而是逐字节从内核拿：

- 每来一个可打印字符（`b >= 0x20`），就回显到串口并写进缓冲区
- `\r` / `\n`：回显换行并结束这一行
- `0x7f` / `0x08`（退格）：如果缓冲区里已有内容，就退一步并输出 `"\x08 \x08"`（典型终端退格效果）
- 其它控制字符（比如 `Ctrl` 组合）会被丢弃

如果串口暂时没有新字节，`read_line` 会得到一个负返回值（常见是 `-11`），然后执行 `sys::yield_now()` 再继续等，因此你看到的体验是“能输入，但系统在忙着切别的任务”。

### 3.1 空格

`catreadme.txt`（没空格）不会被识别成 `cat`。  
`run  hello`（多空格）——`trim` 只剥两端，中间依赖你写对；`run ` 后面再 trim path。

### 3.2 路径

`cat /version` 与 `cat version` 通常都行（内核 `normalize` 剥 `/`）。

### 3.3 阻塞

`run` / `hello` / `badapple` 会占用 shell，直到子任务 `SYS_EXIT` 并被 `wait` 收尸。  
这是刻意的「前台」模型，不是 bug。

更“源码向”的理解：shell 的 `run_path` 逻辑大致是：

```text
id = sys::exec(path)
if id < 0: 直接打印 exec failed
else:
  循环：
    st = sys::wait(id)
    若 st == -11（ERR_AGAIN）: yield_now() 再 poll
    否则：子任务已退出（或已被错误状态收掉），跳出循环
```

这也解释了为什么长任务（如 badapple）期间提示符不返回：shell 自己就在前台 `wait` 里轮询/让出 CPU。

### 3.4 缓冲区

一行最多约 64 字节（`buf = [0u8; 64]`）。别贴超长命令；超过缓冲区上限的部分不会再被加入到这一行里。

## 4. 和 Linux shell 的对照（防脑补）

| Linux 习惯 | DeepRoot shell |
|---|---|
| `ls /bin` | embed 表 + DRFS 目录，无真实 Linux 目录树 |
| `./a.out` | 用 `run name`；ELF 目前只来自 embed `FILES` |
| `Ctrl-C` 杀进程 | 未实现；长任务用程序自己的退出键或重启 QEMU |
| 管道 `\|` | 无 |

下一章：[自己写一个用户程序](write-user-prog.md)。
