// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

const INIT_SCRIPT: &str = r#"
(function() {
    'use strict';

    function hideDownloadPrompts() {
        // Hide elements whose text content is exactly "下载豆包电脑版"
        for (const el of document.querySelectorAll('button, div')) {
            const text = (el.textContent || '').trim();
            if (text === '下载豆包电脑版' || text === '下载电脑版') {
                // Walk up to find the container and hide it
                let p = el;
                for (let i = 0; i < 6 && p && p !== document.body; i++) {
                    p.style.display = 'none';
                    // Also try to remove if it's the outermost container
                    if (p.className && typeof p.className === 'string' &&
                        p.className.includes('container-')) {
                        p.remove();
                        break;
                    }
                    p = p.parentElement;
                }
            }
        }
    }

    if (document.body) hideDownloadPrompts();

    // Watch for dynamic DOM changes
    const observer = new MutationObserver(hideDownloadPrompts);
    observer.observe(document.documentElement, {
        childList: true, subtree: true
    });
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
