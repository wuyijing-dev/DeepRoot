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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">前言</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">入门</li><li class="chapter-item expanded "><a href="intro/what-is-deeproot.html">这是什么？</a></li><li class="chapter-item expanded "><a href="intro/prerequisites.html">你需要准备什么</a></li><li class="chapter-item expanded "><a href="intro/first-boot.html">第一次启动</a></li><li class="chapter-item expanded "><a href="intro/repo-map.html">仓库长什么样</a></li><li class="chapter-item expanded affix "><li class="part-title">跟着版本学</li><li class="chapter-item expanded "><a href="path/overview.html">学习路线图</a></li><li class="chapter-item expanded "><a href="path/01-boot.html">0.1 启动与串口</a></li><li class="chapter-item expanded "><a href="path/02-mm.html">0.2 内存与页表</a></li><li class="chapter-item expanded "><a href="path/03-cap-ipc.html">0.3–0.4 能力与 IPC</a></li><li class="chapter-item expanded "><a href="path/04-user-sched.html">0.5–0.6 用户态与调度</a></li><li class="chapter-item expanded "><a href="path/05-abi.html">1.0 冻结 ABI</a></li><li class="chapter-item expanded "><a href="path/06-as-spawn.html">1.1 地址空间与 spawn</a></li><li class="chapter-item expanded "><a href="path/07-shell.html">1.2 Shell</a></li><li class="chapter-item expanded "><a href="path/08-fs.html">1.3 ramfs 与 run</a></li><li class="chapter-item expanded "><a href="path/09-block.html">1.4 块设备（教学替身）</a></li><li class="chapter-item expanded affix "><li class="part-title">动手玩</li><li class="chapter-item expanded "><a href="hands-on/shell-commands.html">Shell 常用命令</a></li><li class="chapter-item expanded "><a href="hands-on/write-user-prog.html">自己写一个用户程序</a></li><li class="chapter-item expanded "><a href="hands-on/faq.html">常见问题</a></li><li class="chapter-item expanded affix "><li class="part-title">附录</li><li class="chapter-item expanded "><a href="appendix/glossary.html">名词表</a></li><li class="chapter-item expanded "><a href="appendix/versions.html">版本与标签</a></li><li class="chapter-item expanded "><a href="appendix/next.html">下一步可以看什么</a></li></ol>';
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
