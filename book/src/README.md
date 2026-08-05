# DeepRoot 学习笔记

> 版本选择：下拉框会跳转到 `docs/<tag>/` 下对应版本的教程站点（例如 `v1.4.1`、`v1.4.0`）。

<div style="margin: 0.6rem 0; padding: 0.6rem; border: 1px solid #ddd; border-radius: 8px;">
  <div style="margin-bottom: 0.4rem; font-weight: 600;">选择版本</div>
  <select id="drp-version-select" style="min-width: 12rem; padding: 0.35rem 0.5rem;"></select>
</div>

<script>
(function () {
  const select = document.getElementById('drp-version-select');
  if (!select) return;

  // 站点固定部署在 /DeepRoot/（见 docs/book.toml 的 site-url）
  fetch('/DeepRoot/versions.json')
    .then(r => r.json())
    .then(list => {
      select.innerHTML = '';
      for (const item of list) {
        const opt = document.createElement('option');
        opt.value = item.version;
        opt.textContent = item.label || item.version;
        select.appendChild(opt);
      }

      // 当前页面在 /DeepRoot/<version>/ 下时，尝试选中对应版本；否则默认选第一个。
      const m = location.pathname.match(/\/DeepRoot\/([^/]+)\//);
      const current = m ? m[1] : null;
      if (current) select.value = current;

      select.addEventListener('change', () => {
        const v = select.value;
        location.href = '/DeepRoot/' + v + '/';
      });
    })
    .catch(() => {
      // 不影响正文：如果 versions.json 没加载到，就保持空选择框。
    });
})();
</script>

欢迎。这份文档写给**第一次接触微内核 / RISC-V / `no_std` Rust** 的读者。

你不需要已经写过操作系统。你只需要：

- 会一点命令行  
- 愿意按步骤敲命令、看输出  
- 卡住时对照 [常见问题](hands-on/faq.md)

## 文档对应哪个版本？

**当前文档默认对齐：`v1.14.3`**（Frame 收官 + userspace virtioblk on hd1）。  
较早快照仍可通过页首选择器打开。

请核对：

- 仓库根目录 [`VERSION`](https://github.com/wuyijing-dev/DeepRoot/blob/main/VERSION) 第一行  
- 启动横幅里的版本号  
- （可选）`git checkout v1.14.3` 与文字严格对齐  

若你跟的是更新的 `main`，以仓库里的 `VERSION` 为准；用选择器切换冻结快照。

## 怎么读？（请按这个顺序）

1. [这是什么？](intro/what-is-deeproot.md) — 建立图像  
2. [你需要准备什么](intro/prerequisites.md) — 装工具  
3. [第一次启动](intro/first-boot.md) — **必须先跑通 QEMU**  
4. [仓库长什么样](intro/repo-map.md) — 建立代码地图  
5. [学习路线图](path/overview.md) 起，从 **0.1 读到 1.8**  
   - 每章固定结构：概念 → 源码跟读 → 动手验证 → 易错点  
   - **1.6** 重点：[设备树与 virtio-blk](path/10-fdt-virtio.md)  
   - **1.7** 重点：[SMP 多 hart](path/11-smp.md)
   - **1.8** 重点：[更完善自研 shell](path/12-shell18.md)  
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

> 在线站点由仓库 `docs/` 目录发布（mdBook 构建产物；Pages 选 main → /docs）。
