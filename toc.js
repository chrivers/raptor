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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="intro/index.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="intro/install.html"><strong aria-hidden="true">2.</strong> Installling Raptor</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">Walkthrough examples</li><li class="chapter-item expanded "><a href="walkthrough/debian/index.html"><strong aria-hidden="true">3.</strong> Debian Liveboot</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="walkthrough/debian/build.html"><strong aria-hidden="true">3.1.</strong> Build filesystem</a></li><li class="chapter-item expanded "><a href="walkthrough/debian/iso.html"><strong aria-hidden="true">3.2.</strong> Generate iso</a></li><li class="chapter-item expanded "><a href="walkthrough/debian/make.html"><strong aria-hidden="true">3.3.</strong> Use raptor-make</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">Learning Raptor</li><li class="chapter-item expanded "><a href="module-name.html"><strong aria-hidden="true">4.</strong> Module names</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="module-name/relative.html"><strong aria-hidden="true">4.1.</strong> Relative</a></li><li class="chapter-item expanded "><a href="module-name/absolute.html"><strong aria-hidden="true">4.2.</strong> Absolute</a></li><li class="chapter-item expanded "><a href="module-name/package.html"><strong aria-hidden="true">4.3.</strong> Package</a></li></ol></li><li class="chapter-item expanded "><a href="instancing.html"><strong aria-hidden="true">5.</strong> Instancing</a></li><li class="chapter-item expanded "><a href="string-escape.html"><strong aria-hidden="true">6.</strong> String escape</a></li><li class="chapter-item expanded "><a href="expressions.html"><strong aria-hidden="true">7.</strong> Expressions</a></li><li class="chapter-item expanded "><a href="file-options.html"><strong aria-hidden="true">8.</strong> File options</a></li><li class="chapter-item expanded "><a href="mount-types.html"><strong aria-hidden="true">9.</strong> Mount types</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="mount-types/simple.html"><strong aria-hidden="true">9.1.</strong> --simple</a></li><li class="chapter-item expanded "><a href="mount-types/file.html"><strong aria-hidden="true">9.2.</strong> --file</a></li><li class="chapter-item expanded "><a href="mount-types/layers.html"><strong aria-hidden="true">9.3.</strong> --layers</a></li><li class="chapter-item expanded "><a href="mount-types/overlay.html"><strong aria-hidden="true">9.4.</strong> --overlay</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded affix "><li class="part-title">Reference manual</li><li class="chapter-item expanded "><a href="make.html"><strong aria-hidden="true">10.</strong> Raptor Make</a></li><li class="chapter-item expanded "><a href="grammar.html"><strong aria-hidden="true">11.</strong> Grammar</a></li><li class="chapter-item expanded "><a href="syntax.html"><strong aria-hidden="true">12.</strong> Instructions</a></li><li><ol class="section"><li class="chapter-item expanded "><div><strong aria-hidden="true">12.1.</strong> Build instructions</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="inst/from.html"><strong aria-hidden="true">12.1.1.</strong> FROM</a></li><li class="chapter-item expanded "><a href="inst/run.html"><strong aria-hidden="true">12.1.2.</strong> RUN</a></li><li class="chapter-item expanded "><a href="inst/env.html"><strong aria-hidden="true">12.1.3.</strong> ENV</a></li><li class="chapter-item expanded "><a href="inst/workdir.html"><strong aria-hidden="true">12.1.4.</strong> WORKDIR</a></li><li class="chapter-item expanded "><a href="inst/write.html"><strong aria-hidden="true">12.1.5.</strong> WRITE</a></li><li class="chapter-item expanded "><a href="inst/mkdir.html"><strong aria-hidden="true">12.1.6.</strong> MKDIR</a></li><li class="chapter-item expanded "><a href="inst/copy.html"><strong aria-hidden="true">12.1.7.</strong> COPY</a></li><li class="chapter-item expanded "><a href="inst/include.html"><strong aria-hidden="true">12.1.8.</strong> INCLUDE</a></li><li class="chapter-item expanded "><a href="inst/render.html"><strong aria-hidden="true">12.1.9.</strong> RENDER</a></li></ol></li><li class="chapter-item expanded "><div><strong aria-hidden="true">12.2.</strong> Run instructions</div></li><li><ol class="section"><li class="chapter-item expanded "><a href="inst/mount.html"><strong aria-hidden="true">12.2.1.</strong> MOUNT</a></li><li class="chapter-item expanded "><a href="inst/entrypoint.html"><strong aria-hidden="true">12.2.2.</strong> ENTRYPOINT</a></li><li class="chapter-item expanded "><a href="inst/cmd.html"><strong aria-hidden="true">12.2.3.</strong> CMD</a></li></ol></li></ol></li></ol>';
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
