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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="user-guide.html"><strong aria-hidden="true">1.</strong> User guide</a></li><li class="chapter-item expanded "><a href="walkthrough/debian/index.html"><strong aria-hidden="true">2.</strong> Walkthrough: Debian Liveboot</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="walkthrough/debian/build.html"><strong aria-hidden="true">2.1.</strong> Build filesystem</a></li><li class="chapter-item expanded "><a href="walkthrough/debian/iso.html"><strong aria-hidden="true">2.2.</strong> Generate iso</a></li><li class="chapter-item expanded "><a href="walkthrough/debian/make.html"><strong aria-hidden="true">2.3.</strong> Use raptor-make</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="reference-manual.html"><strong aria-hidden="true">3.</strong> Reference manual</a></li><li class="chapter-item expanded "><div><strong aria-hidden="true">4.</strong> Concepts</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="module-name.html"><strong aria-hidden="true">4.1.</strong> Module names</a></li><li class="chapter-item expanded "><a href="string-escape.html"><strong aria-hidden="true">4.2.</strong> String escape</a></li><li class="chapter-item expanded "><a href="expressions.html"><strong aria-hidden="true">4.3.</strong> Expressions</a></li><li class="chapter-item expanded "><a href="file-options.html"><strong aria-hidden="true">4.4.</strong> File options</a></li></ol></li><li class="chapter-item expanded "><a href="syntax.html"><strong aria-hidden="true">5.</strong> Instructions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="inst/from.html"><strong aria-hidden="true">5.1.</strong> FROM</a></li><li class="chapter-item expanded "><a href="inst/run.html"><strong aria-hidden="true">5.2.</strong> RUN</a></li><li class="chapter-item expanded "><a href="inst/env.html"><strong aria-hidden="true">5.3.</strong> ENV</a></li><li class="chapter-item expanded "><a href="inst/workdir.html"><strong aria-hidden="true">5.4.</strong> WORKDIR</a></li><li class="chapter-item expanded "><a href="inst/write.html"><strong aria-hidden="true">5.5.</strong> WRITE</a></li><li class="chapter-item expanded "><a href="inst/mkdir.html"><strong aria-hidden="true">5.6.</strong> MKDIR</a></li><li class="chapter-item expanded "><a href="inst/copy.html"><strong aria-hidden="true">5.7.</strong> COPY</a></li><li class="chapter-item expanded "><a href="inst/include.html"><strong aria-hidden="true">5.8.</strong> INCLUDE</a></li><li class="chapter-item expanded "><a href="inst/render.html"><strong aria-hidden="true">5.9.</strong> RENDER</a></li><li class="chapter-item expanded "><a href="inst/mount.html"><strong aria-hidden="true">5.10.</strong> MOUNT</a></li><li class="chapter-item expanded "><a href="inst/entrypoint.html"><strong aria-hidden="true">5.11.</strong> ENTRYPOINT</a></li><li class="chapter-item expanded "><a href="inst/cmd.html"><strong aria-hidden="true">5.12.</strong> CMD</a></li></ol></li><li class="chapter-item expanded "><a href="make.html"><strong aria-hidden="true">6.</strong> Raptor Make</a></li><li class="chapter-item expanded "><a href="grammar.html"><strong aria-hidden="true">7.</strong> Grammar</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
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
