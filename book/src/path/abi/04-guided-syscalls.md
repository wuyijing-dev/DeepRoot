# 1.0.4 四个 syscall 实战跟读

这一页只做一件事：**带你把 4 个最常见 syscall 从用户态一路跟到内核分支。**

## 1. `SYS_DEBUG_WRITE`

用户态：

```text
sys::debug_write("hello")
  -> ecall(SYS_DEBUG_WRITE, ptr, len, 0, 0)
```

内核态：

```text
trap_handler
  -> handle_syscall(SYS_DEBUG_WRITE, ptr, len, ...)
  -> 长度检查
  -> console::write_bytes(slice)
  -> 返回 len
```

要点：成功时返回写出长度，不只是 0。

## 2. `SYS_DEBUG_READ`

用户态：

```text
sys::debug_read_byte()
```

内核态：

```text
sbi::console_getchar()
  有字节 -> 返回字节值
  没字节 -> ERR_AGAIN
```

要点：shell 必须自己处理 `-11`，并 `yield_now()` 再读。

## 3. `SYS_EXEC`

用户态：

```text
sys::exec(path)
```

内核态大意：

1. 读 `path ptr + len`
2. 校验长度
3. 解析 UTF-8
4. `fs::lookup(path)`
5. 校验是不是 ELF
6. `spawn_elf_bytes(...)`
7. 返回 child sched id

所以 `exec failed` 的根因可能是：

- path 错
- 文件存在但不是 ELF
- ELF 太大
- 地址空间/映射失败

## 4. `SYS_WAIT`

用户态：

```text
sys::wait(id)
```

内核态：

```text
Zombie -> 回收槽位并返回 0
Empty  -> ERR_GENERIC
其他   -> ERR_AGAIN
```

所以 shell 的前台等待模型实际上是：

```text
wait
如果 -11 就 yield
再 wait
```

## 5. 自测问题

1. 为什么 `SYS_DEBUG_READ` 的 `-11` 不该直接当成致命错误？  
2. 为什么 `SYS_EXEC` 成功返回的是 sched id 而不是布尔值？  
3. 为什么 `SYS_WAIT` 成功返回 `0` 也足够有意义？  

下一章：[1.1 地址空间与 spawn](../06-as-spawn.md)

