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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="selene.html"><strong aria-hidden="true">1.</strong> selene</a></li><li class="chapter-item expanded "><a href="motivation.html"><strong aria-hidden="true">2.</strong> Motivation</a></li><li class="chapter-item expanded "><a href="luacheck.html"><strong aria-hidden="true">3.</strong> Luacheck Comparison</a></li><li class="chapter-item expanded "><a href="cli/index.html"><strong aria-hidden="true">4.</strong> Command Line Interface</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="cli/installation.html"><strong aria-hidden="true">4.1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="cli/usage.html"><strong aria-hidden="true">4.2.</strong> CLI Usage</a></li></ol></li><li class="chapter-item expanded "><a href="usage/index.html"><strong aria-hidden="true">5.</strong> Usage</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="usage/configuration.html"><strong aria-hidden="true">5.1.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="usage/filtering.html"><strong aria-hidden="true">5.2.</strong> Filtering</a></li><li class="chapter-item expanded "><a href="usage/std.html"><strong aria-hidden="true">5.3.</strong> Standard Library Format</a></li></ol></li><li class="chapter-item expanded "><a href="roblox.html"><strong aria-hidden="true">6.</strong> Roblox Guide</a></li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">7.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="lints/index.html"><strong aria-hidden="true">8.</strong> Lints</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="lints/almost_swapped.html"><strong aria-hidden="true">8.1.</strong> almost_swapped</a></li><li class="chapter-item expanded "><a href="lints/constant_table_comparison.html"><strong aria-hidden="true">8.2.</strong> constant_table_comparison</a></li><li class="chapter-item expanded "><a href="lints/deprecated.html"><strong aria-hidden="true">8.3.</strong> deprecated</a></li><li class="chapter-item expanded "><a href="lints/divide_by_zero.html"><strong aria-hidden="true">8.4.</strong> divide_by_zero</a></li><li class="chapter-item expanded "><a href="lints/duplicate_keys.html"><strong aria-hidden="true">8.5.</strong> duplicate_keys</a></li><li class="chapter-item expanded "><a href="lints/empty_if.html"><strong aria-hidden="true">8.6.</strong> empty_if</a></li><li class="chapter-item expanded "><a href="lints/empty_loop.html"><strong aria-hidden="true">8.7.</strong> empty_loop</a></li><li class="chapter-item expanded "><a href="lints/global_usage.html"><strong aria-hidden="true">8.8.</strong> global_usage</a></li><li class="chapter-item expanded "><a href="lints/high_cyclomatic_complexity.html"><strong aria-hidden="true">8.9.</strong> high_cyclomatic_complexity</a></li><li class="chapter-item expanded "><a href="lints/if_same_then_else.html"><strong aria-hidden="true">8.10.</strong> if_same_then_else</a></li><li class="chapter-item expanded "><a href="lints/ifs_same_cond.html"><strong aria-hidden="true">8.11.</strong> ifs_same_cond</a></li><li class="chapter-item expanded "><a href="lints/incorrect_standard_library_use.html"><strong aria-hidden="true">8.12.</strong> incorrect_standard_library_use</a></li><li class="chapter-item expanded "><a href="lints/manual_table_clone.html"><strong aria-hidden="true">8.13.</strong> manual_table_clone</a></li><li class="chapter-item expanded "><a href="lints/mismatched_arg_count.html"><strong aria-hidden="true">8.14.</strong> mismatched_arg_count</a></li><li class="chapter-item expanded "><a href="lints/mixed_table.html"><strong aria-hidden="true">8.15.</strong> mixed_table</a></li><li class="chapter-item expanded "><a href="lints/multiple_statements.html"><strong aria-hidden="true">8.16.</strong> multiple_statements</a></li><li class="chapter-item expanded "><a href="lints/must_use.html"><strong aria-hidden="true">8.17.</strong> must_use</a></li><li class="chapter-item expanded "><a href="lints/parenthese_conditions.html"><strong aria-hidden="true">8.18.</strong> parenthese_conditions</a></li><li class="chapter-item expanded "><a href="lints/roblox_incorrect_color3_new_bounds.html"><strong aria-hidden="true">8.19.</strong> roblox_incorrect_color3_new_bounds</a></li><li class="chapter-item expanded "><a href="lints/roblox_incorrect_roact_usage.html"><strong aria-hidden="true">8.20.</strong> roblox_incorrect_roact_usage</a></li><li class="chapter-item expanded "><a href="lints/roblox_manual_fromscale_or_fromoffset.html"><strong aria-hidden="true">8.21.</strong> roblox_manual_fromscale_or_fromoffset</a></li><li class="chapter-item expanded "><a href="lints/roblox_suspicious_udim2_new.html"><strong aria-hidden="true">8.22.</strong> roblox_suspicious_udim2_new</a></li><li class="chapter-item expanded "><a href="lints/shadowing.html"><strong aria-hidden="true">8.23.</strong> shadowing</a></li><li class="chapter-item expanded "><a href="lints/suspicious_reverse_loop.html"><strong aria-hidden="true">8.24.</strong> suspicious_reverse_loop</a></li><li class="chapter-item expanded "><a href="lints/type_check_inside_call.html"><strong aria-hidden="true">8.25.</strong> type_check_inside_call</a></li><li class="chapter-item expanded "><a href="lints/unbalanced_assignments.html"><strong aria-hidden="true">8.26.</strong> unbalanced_assignments</a></li><li class="chapter-item expanded "><a href="lints/undefined_variable.html"><strong aria-hidden="true">8.27.</strong> undefined_variable</a></li><li class="chapter-item expanded "><a href="lints/unscoped_variables.html"><strong aria-hidden="true">8.28.</strong> unscoped_variables</a></li><li class="chapter-item expanded "><a href="lints/unused_variable.html"><strong aria-hidden="true">8.29.</strong> unused_variable</a></li></ol></li><li class="chapter-item expanded "><a href="archive/index.html"><strong aria-hidden="true">9.</strong> Archive</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="archive/std_v1.html"><strong aria-hidden="true">9.1.</strong> TOML Standard Library Format</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
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
