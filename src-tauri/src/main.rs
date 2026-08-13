// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

const INIT_SCRIPT: &str = r#"
(function() {
    // CJK font rendering tweaks for WebKitGTK
    var s = document.createElement('style');
    s.textContent = '*{font-family:"Douyin Sans","抖音美好体",'
        +'"PingFang SC","Noto Sans CJK SC","Source Han Sans CN",'
        +'"Microsoft YaHei","WenQuanYi Micro Hei",'
        +'sans-serif!important;-webkit-font-smoothing:antialiased!important}';
    document.head.appendChild(s);

    // Hide download prompt button
    function hide() {
        var btns = document.querySelectorAll('button');
        for (var i = 0; i < btns.length; i++) {
            if (btns[i].textContent.trim() === '下载豆包电脑版') {
                btns[i].style.display = 'none';
                clearInterval(tid);
                return;
            }
        }
    }
    var tid = setInterval(hide, 1000);
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
