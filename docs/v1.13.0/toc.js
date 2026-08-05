// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">前言</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">入门</li><li class="chapter-item expanded "><a href="intro/what-is-deeproot.html">这是什么？</a></li><li class="chapter-item expanded "><a href="intro/prerequisites.html">你需要准备什么</a></li><li class="chapter-item expanded "><a href="intro/first-boot.html">第一次启动</a></li><li class="chapter-item expanded "><a href="intro/repo-map.html">仓库长什么样</a></li><li class="chapter-item expanded affix "><li class="part-title">跟着版本学</li><li class="chapter-item expanded "><a href="path/overview.html">学习路线图</a></li><li class="chapter-item expanded "><a href="path/01-boot.html">0.1 启动与串口</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/boot/01-boot-path.html">0.1.1 从 QEMU 到 _start</a></li><li class="chapter-item expanded "><a href="path/boot/02-boot-rs.html">0.1.2 跟读 boot.rs</a></li><li class="chapter-item expanded "><a href="path/boot/03-kernel-main.html">0.1.3 跟读 kernel_main</a></li><li class="chapter-item expanded "><a href="path/boot/04-sbi-console.html">0.1.4 SBI 控制台</a></li><li class="chapter-item expanded "><a href="path/boot/05-early-trap.html">0.1.5 early trap 与 stvec</a></li></ol></li><li class="chapter-item expanded "><a href="path/02-mm.html">0.2 内存与页表</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/mm/01-memory-map.html">0.2.1 RAM 发现与 DTB 回退</a></li><li class="chapter-item expanded "><a href="path/mm/02-frame-heap.html">0.2.2 frame allocator 与 heap</a></li><li class="chapter-item expanded "><a href="path/mm/03-sv39.html">0.2.3 Sv39 身份映射</a></li><li class="chapter-item expanded "><a href="path/mm/04-elf-preview.html">0.2.4 为什么这决定 ELF 装载</a></li></ol></li><li class="chapter-item expanded "><a href="path/03-cap-ipc.html">0.3–0.4 能力与 IPC</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/cap-ipc/01-cap-model.html">0.3.1 Capability 模型</a></li><li class="chapter-item expanded "><a href="path/cap-ipc/02-boot-cspace.html">0.3.2 启动时的 CSpace 安装</a></li><li class="chapter-item expanded "><a href="path/cap-ipc/03-ipc-call-flow.html">0.4.1 call / recv / reply</a></li><li class="chapter-item expanded "><a href="path/cap-ipc/04-ledger.html">0.4.2 Root Ledger 怎么看</a></li><li class="chapter-item expanded "><a href="path/cap-ipc/05-ipc-sched.html">0.4.3 IPC 与调度状态切换</a></li></ol></li><li class="chapter-item expanded "><a href="path/04-user-sched.html">0.5–0.6 用户态与调度</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/user-sched/01-user-runtime.html">0.5.1 用户程序最小骨架</a></li><li class="chapter-item expanded "><a href="path/user-sched/02-bring-up.html">0.5.2 servers::bring_up 跟读</a></li><li class="chapter-item expanded "><a href="path/user-sched/03-task-states.html">0.6.1 TaskState 与 BlockReason</a></li><li class="chapter-item expanded "><a href="path/user-sched/04-timer-preempt.html">0.6.2 timer / preempt</a></li><li class="chapter-item expanded "><a href="path/user-sched/05-syscall-return.html">0.6.3 syscall 返回值到底写给谁</a></li></ol></li><li class="chapter-item expanded "><a href="path/05-abi.html">1.0 冻结 ABI</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/abi/01-registers.html">1.0.1 寄存器调用约定</a></li><li class="chapter-item expanded "><a href="path/abi/02-errors-core-syscalls.html">1.0.2 错误码与核心 syscall</a></li><li class="chapter-item expanded "><a href="path/abi/03-trap-decode.html">1.0.3 trap.rs 如何解码 ecall</a></li><li class="chapter-item expanded "><a href="path/abi/04-guided-syscalls.html">1.0.4 四个 syscall 实战跟读</a></li></ol></li><li class="chapter-item expanded "><a href="path/06-as-spawn.html">1.1 地址空间与 spawn</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/as-spawn/01-per-task-as.html">1.1.1 per-task 页表</a></li><li class="chapter-item expanded "><a href="path/as-spawn/02-sys-spawn.html">1.1.2 SYS_SPAWN 控制流</a></li><li class="chapter-item expanded "><a href="path/as-spawn/03-elf-loader.html">1.1.3 跟读 elf.rs</a></li><li class="chapter-item expanded "><a href="path/as-spawn/04-zombie-wait.html">1.1.4 Zombie 与 SYS_WAIT</a></li></ol></li><li class="chapter-item expanded "><a href="path/07-shell.html">1.2 Shell</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/shell/01-main-loop.html">1.2.1 shell 主循环</a></li><li class="chapter-item expanded "><a href="path/shell/02-read-line.html">1.2.2 read_line 与共享串口</a></li><li class="chapter-item expanded "><a href="path/shell/03-run-path.html">1.2.3 run_path 与前台等待</a></li></ol></li><li class="chapter-item expanded "><a href="path/08-fs.html">1.3 ramfs 与 run</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/fs/01-ramfs-model.html">1.3.1 ramfs 模型</a></li><li class="chapter-item expanded "><a href="path/fs/02-build-pipeline.html">1.3.2 build.rs 如何产出嵌入字节</a></li><li class="chapter-item expanded "><a href="path/fs/03-fs-syscalls.html">1.3.3 FS_LIST / FS_CAT / EXEC</a></li><li class="chapter-item expanded "><a href="path/fs/04-spawn-vs-exec.html">1.3.4 SYS_SPAWN vs SYS_EXEC</a></li></ol></li><li class="chapter-item expanded "><a href="path/09-block.html">1.4 块设备（教学替身）</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/block/01-why-standin.html">1.4.1 为什么先做替身</a></li><li class="chapter-item expanded "><a href="path/block/02-read-block-rs.html">1.4.2 跟读 block.rs</a></li><li class="chapter-item expanded "><a href="path/block/03-next-step-virtio.html">1.4.3 走向 virtio（见 1.5–1.6）</a></li></ol></li><li class="chapter-item expanded "><a href="path/10-fdt-virtio.html">1.5–1.6 设备树与 virtio-blk</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/fdt-virtio/01-own-dts.html">1.6.1 自有设备树 deeproot.dts</a></li><li class="chapter-item expanded "><a href="path/fdt-virtio/02-fdt-walker.html">1.5 跟读 fdt.rs</a></li><li class="chapter-item expanded "><a href="path/fdt-virtio/03-virtio-blk.html">1.6 virtio-blk 与 DRFS 后端</a></li></ol></li><li class="chapter-item expanded "><a href="path/11-smp.html">1.7 SMP 多 hart</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/smp/01-hsm-bringup.html">1.7.1 HSM 拉起二级核</a></li><li class="chapter-item expanded "><a href="path/smp/02-per-hart-rq.html">1.7.2 每 hart 运行队列与 idle</a></li><li class="chapter-item expanded "><a href="path/smp/03-locks-ipi.html">1.7.3 锁、IPI 与 tp 陷阱</a></li></ol></li><li class="chapter-item expanded "><a href="path/12-shell18.html">1.8 更完善自研 shell</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/shell18/01-parser-history.html">1.8.1 解析器与 history</a></li><li class="chapter-item expanded "><a href="path/shell18/02-env-cd-bg.html">1.8.2 env / cd / 后台</a></li><li class="chapter-item expanded "><a href="path/shell18/03-pipe-redir.html">1.8.3 管道与重定向</a></li></ol></li><li class="chapter-item expanded "><a href="path/13-fs19.html">1.9 文件系统加深</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/fs19/01-vfs-tree.html">1.9.1 VFS 树与路径</a></li><li class="chapter-item expanded "><a href="path/fs19/02-syscalls-shell.html">1.9.2 syscall 与 shell 内建</a></li></ol></li><li class="chapter-item expanded "><a href="path/14-modules.html">1.10 可加载模块</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="path/modules/01-spawn-server.html">1.10.1 SYS_SPAWN_SERVER</a></li><li class="chapter-item expanded "><a href="path/modules/02-moddemo-init.html">1.10.2 moddemo 与 init</a></li></ol></li><li class="chapter-item expanded "><a href="path/15-fs11.html">1.11 文件系统持久化</a></li><li class="chapter-item expanded "><a href="path/16-lab12.html">1.11–1.12 fd 与实用工具</a></li><li class="chapter-item expanded "><a href="path/17-svc13.html">1.13 服务命名 / 发现</a></li><li class="chapter-item expanded affix "><li class="part-title">动手玩</li><li class="chapter-item expanded "><a href="hands-on/shell-commands.html">Shell 常用命令</a></li><li class="chapter-item expanded "><a href="hands-on/write-user-prog.html">自己写一个用户程序</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="hands-on/write-user-prog/01-clone-template.html">步骤 1：复制 hello 模板</a></li><li class="chapter-item expanded "><a href="hands-on/write-user-prog/02-register-build.html">步骤 2：注册到 workspace 与 build</a></li><li class="chapter-item expanded "><a href="hands-on/write-user-prog/03-ramfs-shell.html">步骤 3：挂进 ramfs 与 shell</a></li><li class="chapter-item expanded "><a href="hands-on/write-user-prog/04-build-debug.html">步骤 4：构建、运行、调试</a></li></ol></li><li class="chapter-item expanded "><a href="hands-on/faq.html">常见问题</a></li><li class="chapter-item expanded affix "><li class="part-title">附录</li><li class="chapter-item expanded "><a href="appendix/glossary.html">名词表</a></li><li class="chapter-item expanded "><a href="appendix/versions.html">版本与标签</a></li><li class="chapter-item expanded "><a href="appendix/next.html">下一步可以看什么</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
