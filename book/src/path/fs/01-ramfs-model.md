# ramfs 模型（与 1.4.1 块文件）

这一页讲：**DeepRoot 的「文件」从哪来，shell 怎么看见它们。**

## 1. 两条来源（1.4.1）

```text
embed ramfs （fs.rs FILES[]）
  ← include_bytes! / 静态字符串
  ← 文本 + ELF（run 只用这里的 ELF）

block DRFS （block.rs DISK[]）
  ← init 时格式化的教学镜像
  ← 目前主要是文本（block.txt 等）
```

`ls` 会打印两节：`fs: ramfs /` 与 `fs: block /`。  
`cat` 先查 embed，未命中再查 DRFS。  
`run` / `SYS_EXEC` **只**走 `fs::lookup`（embed）。

## 2. embed 表长什么样

```text
名字 -> &'static [u8]
```

例如 `version`、`readme.txt`、`hello`（ELF）。  
构建时 `kernel/build.rs` 把用户程序编成字节，再 `include_bytes!` 嵌进内核。

## 3. 它仍然不是

- 可拔插的真实磁盘（后端是内存 ramdisk）  
- 用户态 VFS / 可写目录树  
- 完整 Unix 权限模型  

## 4. 为什么教学上拆成两层

1. **1.3**：先学会「路径 → 字节 → 显示或执行」（纯 embed）。  
2. **1.4.0**：块层模块出现在启动路径上。  
3. **1.4.1**：同一套 `ls`/`cat` 开始读块上布局，为以后换 virtio 留接口形状。

下一页：[build.rs 如何产出嵌入字节](02-build-pipeline.md)
