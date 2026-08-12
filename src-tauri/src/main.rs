// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

const INIT_SCRIPT: &str = r#"
(function() {
    'use strict';

    // CSS to hide desktop download prompts & buttons
    const HIDE_CSS = `
        /* Hide common download/app promotion elements */
        [class*="download" i],
        [class*="Download" i],
        [id*="download" i],
        [id*="Download" i],
        [class*="app-download" i],
        [class*="desktop-download" i],
        [class*="download-app" i],
        [class*="download-client" i],
        [class*="client-download" i],
        [class*="pc-download" i],
        [class*="app-promotion" i],
        [class*="open-app" i],
        [class*="app-guide" i],
        [class*="app-banner" i],
        [class*="app-entry" i],
        [class*="desktop-tip" i],
        [class*="desktop-banner" i],
        [class*="desktop-bar" i],
        [class*="app-bar" i],
        [class*="download-bar" i],
        [class*="bottom-bar" i],
        [class*="bottom-download" i],
        [class*="sidebar-download" i],
        a[href*="download"],
        a[href*="/desktop"],
        a[href*="/app"]
        { display: none !important; visibility: hidden !important;
          opacity: 0 !important; pointer-events: none !important;
          height: 0 !important; width: 0 !important; overflow: hidden !important; }

        /* Hide elements whose text contains download keywords */
        [aria-label*="下载" i],
        [aria-label*="桌面" i],
        [title*="下载" i],
        [title*="桌面版" i],
        [title*="客户端" i]
        { display: none !important; }
    `;

    const style = document.createElement('style');
    style.id = 'doubao-hide-download';
    style.textContent = HIDE_CSS;
    (document.head || document.documentElement).appendChild(style);

    // MutationObserver: catch dynamically added elements with download text
    const DOWNLOAD_KEYWORDS = ['下载电脑版','下载桌面版','下载客户端',
        '下载桌面客户端','下载APP','下载 App','下载 app',
        '电脑版下载','桌面版下载','客户端下载',
        'desktop app','download desktop','get the app'];

    function shouldHide(el) {
        const text = (el.textContent || '').trim();
        if (text.length > 30) return false;
        return DOWNLOAD_KEYWORDS.some(k => text.includes(k));
    }

    function hideDownloadElements() {
        // Walk all small text elements that might be download buttons/prompts
        const walker = document.createTreeWalker(
            document.body,
            NodeFilter.SHOW_ELEMENT,
            {
                acceptNode: function(node) {
                    if (node.children.length > 0) return NodeFilter.FILTER_SKIP;
                    if (shouldHide(node)) return NodeFilter.FILTER_ACCEPT;
                    return NodeFilter.FILTER_SKIP;
                }
            }
        );
        while (walker.nextNode()) {
            const el = walker.currentNode;
            let p = el;
            // Walk up to find a hideable container (up to 5 levels)
            for (let i = 0; i < 5 && p && p !== document.body; i++) {
                p.style.display = 'none';
                p = p.parentElement;
            }
        }
    }

    // Run on load and whenever DOM changes
    if (document.body) {
        hideDownloadElements();
    }
    const observer = new MutationObserver(() => hideDownloadElements());
    observer.observe(document.documentElement || document.body, {
        childList: true, subtree: true, characterData: true
    });
    // Re-run periodically for the first 10 seconds to catch late renders
    let runs = 0;
    const interval = setInterval(() => {
        hideDownloadElements();
        if (++runs > 20) clearInterval(interval);
    }, 500);
})();
"#;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let _window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(
                    "https://www.doubao.com/chat/"
                        .parse()
                        .unwrap(),
                ),
            )
            .title("豆包")
            .inner_size(1200.0, 800.0)
            .min_inner_size(600.0, 400.0)
            .initialization_script(INIT_SCRIPT)
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running doubao-desktop");
}
