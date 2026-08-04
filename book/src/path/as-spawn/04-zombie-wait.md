# 1.1.4 Zombie 与 `SYS_WAIT`

这一页只讲：**为什么任务 exit 以后不会立刻完全消失。**

## 1. Zombie 是什么

在 DeepRoot 里，任务 `SYS_EXIT` 之后通常不是“当场彻底删掉”，而是先进入：

```text
TaskState::Zombie
```

这样 shell 或父任务还有机会通过 `SYS_WAIT` 观察并回收它。

## 2. `SYS_WAIT` 的语义

大意是：

- `Zombie` → 回收并返回 `0`
- 还在跑 → `ERR_AGAIN`
- 根本无此任务 / 槽已空 → `ERR_GENERIC`

## 3. 为什么这和 shell 强相关

shell 的 `run` 是前台模型：

```text
exec(path) -> id
while wait(id) == -11:
  yield
```

所以：

- 任务退出太快，shell 很快回提示符
- 任务没退出，shell 就一直等
- 任务已经被回收，再 `wait` 就不是成功路径了

## 4. 最小实验

1. 连续 `run hello` 两次，确认第二次仍能成功。  
2. 故意让一个用户程序不 exit，观察 shell 为什么不返回提示符。  

下一章：[1.2 Shell](../07-shell.md)

