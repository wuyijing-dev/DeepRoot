# Shell 常用命令（详细手册）

提示符：`deeproot>`  
解析器在 `user/shell/src/main.rs`：非常朴素——**没有**引号、管道、变量、通配符。

## 1. 命令一览

| 你输入 | 实际行为 | 底层 |
|---|---|---|
| `help` | 打印内置帮助文本 | 仅用户态字符串 |
| `ls` | 列出 ramfs | `SYS_FS_LIST` |
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
deeproot> cat hello
```

期望：文本正常；对 `hello` 提示用 `run`，不要刷二进制。

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

### 3.1 空格

`catreadme.txt`（没空格）不会被识别成 `cat`。  
`run  hello`（多空格）——`trim` 只剥两端，中间依赖你写对；`run ` 后面再 trim path。

### 3.2 路径

`cat /version` 与 `cat version` 通常都行（内核 `normalize` 剥 `/`）。

### 3.3 阻塞

`run` / `hello` / `badapple` 会占用 shell，直到子任务 `SYS_EXIT` 并被 `wait` 收尸。  
这是刻意的「前台」模型，不是 bug。

### 3.4 缓冲区

一行最多约 64 字节（`buf = [0u8; 64]`）。别贴超长命令。

## 4. 和 Linux shell 的对照（防脑补）

| Linux 习惯 | DeepRoot shell |
|---|---|
| `ls /bin` | 只有内嵌 ramfs，无真实目录树 |
| `./a.out` | 用 `run name`，name 来自 `FILES` |
| `Ctrl-C` 杀进程 | 未实现；长任务用程序自己的退出键或重启 QEMU |
| 管道 `\|` | 无 |

下一章：[自己写一个用户程序](write-user-prog.md)。
