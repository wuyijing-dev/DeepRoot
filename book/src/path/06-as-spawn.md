# 1.1 地址空间与 spawn（详细跟读）

## 本章拆读顺序

1. [1.1.1 per-task 页表](as-spawn/01-per-task-as.md)
2. [1.1.2 `SYS_SPAWN` 控制流](as-spawn/02-sys-spawn.md)
3. [1.1.3 跟读 `elf.rs`](as-spawn/03-elf-loader.md)
4. [1.1.4 Zombie 与 `SYS_WAIT`](as-spawn/04-zombie-wait.md)

## 1. 为什么要「每任务一张页表」？

若所有用户程序共用内核那套「大一统」映射，隔离很弱，一个程序乱写容易牵连别人。  
1.1 起：

- 每个任务有自己的 **根页表**（`UserTask.root_pa` / `AddrSpace`）  
- 调度切换时调度器会调用 `UserTask.activate_as()`：如果 `root_pa != 0`，就用 `sv39::activate(root_pa)` 切换到该任务的根页表环境（也就是“真正打开了它的地址空间”）  
- ELF 加载进**该任务**的虚址空间  

这是现代 OS「进程」直觉的教学版（仍然没有 Linux 那套 fd/信号）。

## 2. `SYS_SPAWN` 逐步发生了什么？

在 `sched.rs` 的 `SYS_SPAWN` 分支（阅读时跟符号）：

1. 按 `blob_id` 选择字节：`0 → servers::HELLO_ELF`  
2. `tasks.spawn` 分配能力系统里的任务对象  
3. 找空闲调度槽，计算栈基址 `next_spawn_stack_base`  
4. `spawn_elf_bytes`：  
   - `AddrSpace::create`  
   - `elf::load_into` 映射 PT_LOAD  
   - 填 trap frame 的 `sepc`（入口）、`sp` 等  
5. 返回 **sched id**（正数）给调用者  

注意：这个 **sched id** 在 `SYS_SPAWN` 里不只是“任务编号”。调度器会先找一个空闲槽位下标 `slot`，再用 `next_spawn_stack_base(slot)` 计算栈虚址基址。也就是说：槽位和栈地址是绑定的，天然减少了任务之间互相覆盖栈的风险。

init 里：

```text
sys::spawn(0)  →  期望看到 hello 输出
```

shell 后来更多用 `SYS_EXEC`，但底层仍走相似的 `spawn_elf_bytes`。

## 3. ELF 加载器你要盯的点（`elf.rs`）

- 校验 magic、`ET_EXEC`、`EM_RISCV`  
- 遍历 program header，处理 `PT_LOAD`  
- 按页分配物理帧，把文件内容拷进页  
- 设置执行/可写等权限  
- **页数上限**（`MAX_PAGES`）：ELF 太大（例如巨大的帧数据）会加载失败 → shell 报 `exec failed`

历史上 badapple 变大时就碰到过上限，需要调大 `MAX_PAGES`。

另一个“会导致诡异 page fault”的细节：ELF 加载器会把多个 `PT_LOAD` 可能落在同一页上的段做**合并**（合并权限位），避免出现“前一步把页映成 R-X，下一段又把它覆盖成 R-W 却不允许执行”的情况。

## 4. 僵尸与等待

任务 `SYS_EXIT` → `Zombie`。  
若不回收，槽位会耗尽。shell 的 `run`：

```text
exec(path) → id
循环: wait(id)；若 ERR_AGAIN 则 yield；否则结束
```

`SYS_WAIT`：若是僵尸则清空槽位返回 0；若还在跑返回 `ERR_AGAIN`。

因此你在 shell 的 `wait` 循环里会看到：当返回值是 `-11`（`ERR_AGAIN`）时，并不是“失败了”，而是“还没退出”。shell 选择 `yield_now()` 让出 CPU，再来 poll 一次。

## 5. 动手验证

1. `run hello` 两次，确认都能成功（说明 wait 回收了槽位）。  
2. 阅读 `user/hello/linker.ld` 的基址，在 `elf` 日志或调试里对照入口。  
3. 把 `SYS_SPAWN` 换成非法 blob id，看返回值。

## 6. 易错点

| 现象 | 原因 |
|---|---|
| `exec failed` | 找不到文件 / 非 ELF / 映射失败 / 页数不足 |
| hello 无输出但返回了 | 调度未运行到它，或输出被刷屏淹没 |
| 第二次 spawn 失败 | 僵尸未回收、槽位满 |

下一章：[1.2 Shell](07-shell.md)。
