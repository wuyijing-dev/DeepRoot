# DeepRoot 学习笔记

欢迎。这份文档写给**第一次接触微内核 / RISC-V / `no_std` Rust** 的读者。

你不需要已经写过操作系统。你只需要：

- 会一点命令行  
- 愿意按步骤敲命令、看输出  
- 卡住时对照 [常见问题](hands-on/faq.md)

## 文档对应哪个版本？

**基线：`v1.4.0`**（教学路径到此封顶）。

请核对：

- 仓库根目录 [`VERSION`](https://github.com/wuyijing-dev/DeepRoot/blob/main/VERSION) 第一行  
- 启动横幅里的版本号  
- （可选）`git checkout v1.4.0` 与文字严格对齐  

若你跟的是更新的 `main`，以仓库里的 `VERSION` 为准。

## 怎么读？（请按这个顺序）

1. [这是什么？](intro/what-is-deeproot.md) — 建立图像  
2. [你需要准备什么](intro/prerequisites.md) — 装工具  
3. [第一次启动](intro/first-boot.md) — **必须先跑通 QEMU**  
4. [仓库长什么样](intro/repo-map.md) — 建立代码地图  
5. [学习路线图](path/overview.md) 起，从 **0.1 读到 1.4**  
   - 每章固定结构：概念 → 源码跟读 → 动手验证 → 易错点  
6. 想创造时看 [动手玩](hands-on/shell-commands.md)

## 每一章你该怎么学？

不要只「看过标题」：

1. 打开章里点名的源码文件  
2. 用编辑器搜索章里出现的符号（`SYS_EXEC`、`bring_up`…）  
3. 做「动手验证」——至少做一半  
4. 现象对不上时先查该章「易错点」，再查 [FAQ](hands-on/faq.md)

## 这份文档不会做什么？

- 不会假装 DeepRoot 兼容 Linux / POSIX  
- 不会把 Bad Apple 之类 demo 当成核心教学内容  
- 不会要求你「先做完习题才能开机」——内核始终是可运行的真内核  

准备好了？从 [这是什么？](intro/what-is-deeproot.md) 开始。

> 在线站点由 `gh-pages` 分支发布（mdBook 构建产物）。
