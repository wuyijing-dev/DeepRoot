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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="index.html">前言</a></span></li><li class="chapter-item expanded "><li class="spacer"></li></li><li class="chapter-item expanded "><li class="part-title">入门</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="intro/what-is-deeproot.html">这是什么？</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="intro/prerequisites.html">你需要准备什么</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="intro/first-boot.html">第一次启动</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="intro/repo-map.html">仓库长什么样</a></span></li><li class="chapter-item expanded "><li class="part-title">跟着版本学</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/overview.html">学习路线图</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/01-boot.html">0.1 启动与串口</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/boot/01-boot-path.html">0.1.1 从 QEMU 到 _start</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/boot/02-boot-rs.html">0.1.2 跟读 boot.rs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/boot/03-kernel-main.html">0.1.3 跟读 kernel_main</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/boot/04-sbi-console.html">0.1.4 SBI 控制台</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/boot/05-early-trap.html">0.1.5 early trap 与 stvec</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/02-mm.html">0.2 内存与页表</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/mm/01-memory-map.html">0.2.1 RAM 发现与 DTB 回退</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/mm/02-frame-heap.html">0.2.2 frame allocator 与 heap</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/mm/03-sv39.html">0.2.3 Sv39 身份映射</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/mm/04-elf-preview.html">0.2.4 为什么这决定 ELF 装载</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/03-cap-ipc.html">0.3–0.4 能力与 IPC</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/cap-ipc/01-cap-model.html">0.3.1 Capability 模型</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/cap-ipc/02-boot-cspace.html">0.3.2 启动时的 CSpace 安装</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/cap-ipc/03-ipc-call-flow.html">0.4.1 call / recv / reply</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/cap-ipc/04-ledger.html">0.4.2 Root Ledger 怎么看</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/cap-ipc/05-ipc-sched.html">0.4.3 IPC 与调度状态切换</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/04-user-sched.html">0.5–0.6 用户态与调度</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/user-sched/01-user-runtime.html">0.5.1 用户程序最小骨架</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/user-sched/02-bring-up.html">0.5.2 servers::bring_up 跟读</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/user-sched/03-task-states.html">0.6.1 TaskState 与 BlockReason</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/user-sched/04-timer-preempt.html">0.6.2 timer / preempt</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/user-sched/05-syscall-return.html">0.6.3 syscall 返回值到底写给谁</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/05-abi.html">1.0 冻结 ABI</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/abi/01-registers.html">1.0.1 寄存器调用约定</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/abi/02-errors-core-syscalls.html">1.0.2 错误码与核心 syscall</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/abi/03-trap-decode.html">1.0.3 trap.rs 如何解码 ecall</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/abi/04-guided-syscalls.html">1.0.4 四个 syscall 实战跟读</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/06-as-spawn.html">1.1 地址空间与 spawn</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/as-spawn/01-per-task-as.html">1.1.1 per-task 页表</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/as-spawn/02-sys-spawn.html">1.1.2 SYS_SPAWN 控制流</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/as-spawn/03-elf-loader.html">1.1.3 跟读 elf.rs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/as-spawn/04-zombie-wait.html">1.1.4 Zombie 与 SYS_WAIT</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/07-shell.html">1.2 Shell</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell/01-main-loop.html">1.2.1 shell 主循环</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell/02-read-line.html">1.2.2 read_line 与共享串口</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell/03-run-path.html">1.2.3 run_path 与前台等待</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/08-fs.html">1.3 ramfs 与 run</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs/01-ramfs-model.html">1.3.1 ramfs 模型</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs/02-build-pipeline.html">1.3.2 build.rs 如何产出嵌入字节</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs/03-fs-syscalls.html">1.3.3 FS_LIST / FS_CAT / EXEC</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs/04-spawn-vs-exec.html">1.3.4 SYS_SPAWN vs SYS_EXEC</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/09-block.html">1.4 块设备（教学替身）</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/block/01-why-standin.html">1.4.1 为什么先做替身</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/block/02-read-block-rs.html">1.4.2 跟读 block.rs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/block/03-next-step-virtio.html">1.4.3 走向 virtio（见 1.5–1.6）</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/10-fdt-virtio.html">1.5–1.6 设备树与 virtio-blk</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fdt-virtio/01-own-dts.html">1.6.1 自有设备树 deeproot.dts</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fdt-virtio/02-fdt-walker.html">1.5 跟读 fdt.rs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fdt-virtio/03-virtio-blk.html">1.6 virtio-blk 与 DRFS 后端</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/11-smp.html">1.7 SMP 多 hart</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/smp/01-hsm-bringup.html">1.7.1 HSM 拉起二级核</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/smp/02-per-hart-rq.html">1.7.2 每 hart 运行队列与 idle</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/smp/03-locks-ipi.html">1.7.3 锁、IPI 与 tp 陷阱</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/12-shell18.html">1.8 更完善自研 shell</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell18/01-parser-history.html">1.8.1 解析器与 history</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell18/02-env-cd-bg.html">1.8.2 env / cd / 后台</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/shell18/03-pipe-redir.html">1.8.3 管道与重定向</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/13-fs19.html">1.9 文件系统加深</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs19/01-vfs-tree.html">1.9.1 VFS 树与路径</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/fs19/02-syscalls-shell.html">1.9.2 syscall 与 shell 内建</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/14-modules.html">1.10 可加载模块</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/modules/01-spawn-server.html">1.10.1 SYS_SPAWN_SERVER</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="path/modules/02-moddemo-init.html">1.10.2 moddemo 与 init</a></span></li></ol><li class="chapter-item expanded "><li class="part-title">动手玩</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/shell-commands.html">Shell 常用命令</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/write-user-prog.html">自己写一个用户程序</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/write-user-prog/01-clone-template.html">步骤 1：复制 hello 模板</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/write-user-prog/02-register-build.html">步骤 2：注册到 workspace 与 build</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/write-user-prog/03-ramfs-shell.html">步骤 3：挂进 ramfs 与 shell</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/write-user-prog/04-build-debug.html">步骤 4：构建、运行、调试</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="hands-on/faq.html">常见问题</a></span></li><li class="chapter-item expanded "><li class="part-title">附录</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/glossary.html">名词表</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/versions.html">版本与标签</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="appendix/next.html">下一步可以看什么</a></span></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            // Check both with and without the '.html' suffix to be robust against pretty URLs
            if (link.href.replace(/\.html$/, '') === current_page.replace(/\.html$/, '')
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();

