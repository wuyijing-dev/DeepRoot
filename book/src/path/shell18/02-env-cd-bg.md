# env / cd / 后台 `&`

对齐：**v1.8.0**。

## 1. 环境变量（shell 本地）

```text
export KEY=VAL
env
echo $KEY
```

存在 shell 进程自己的表里；**不会**自动传给子 ELF（没有 POSIX `environ` 传递）。  
`echo` 会对参数做 `$NAME` 展开。

预置示例：`SHELL=deeproot`、`VERSION=1.8.0`。

## 2. cd / pwd

教学 ramfs 仍是**扁平**名字空间。`cd` 只维护 shell 侧前缀，供 `cat` / `run` 拼相对路径：

```text
pwd          → / 或当前前缀
cd notes     → 之后 cat x ≈ notes/x（拼接）
cd /         → 清空前缀
```

没有真正的目录 inode；这是为满足路线图「有 cwd 语义」的最小实现。

## 3. 后台 `&`

```text
run hello &
```

- `SYS_EXEC` 后**不** `wait`  
- 打印 `shell: [bg] id=N`  
- 前台 `run hello` 仍会 wait，直到子任务 exit（长任务如 badapple 仍占串口）

## 4. 动手

```text
export A=1
echo $A
pwd
cd demo
pwd
run hello &
```
