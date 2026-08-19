// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[derive(serde::Serialize)]
struct ClipboardImage {
    width: usize,
    height: usize,
    /// RGBA8 pixel data, base64-encoded.
    data: String,
}

/// Read the raw image from the system clipboard as RGBA8.
///
/// WebKitGTK does not deliver image data through the DOM `paste` event, so we
/// read it natively and re-inject it into the page as a synthetic paste event.
#[tauri::command]
fn get_clipboard_image() -> Result<Option<ClipboardImage>, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    match clipboard.get_image() {
        Ok(img) => Ok(Some(ClipboardImage {
            width: img.width,
            height: img.height,
            data: STANDARD.encode(img.bytes.as_ref()),
        })),
        // No image on the clipboard (e.g. plain text) — not an error.
        Err(_) => Ok(None),
    }
}

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

    // --- Image paste workaround for WebKitGTK ---
    // WebKitGTK drops image data from the DOM `paste` event's clipboardData.
    // Read the native clipboard via Tauri and re-inject a synthetic paste.
    if (window.__TAURI_INTERNALS__) {
        function b64ToBytes(b64) {
            var bin = atob(b64);
            var bytes = new Uint8Array(bin.length);
            for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
            return bytes;
        }

        document.addEventListener('paste', function(e) {
            // If the event already carries an image, leave it alone.
            var items = e.clipboardData && e.clipboardData.items;
            if (items) {
                for (var i = 0; i < items.length; i++) {
                    if (items[i].type && items[i].type.indexOf('image') === 0) return;
                }
            }

            window.__TAURI_INTERNALS__.invoke('get_clipboard_image')
                .then(function(img) {
                    if (!img) return; // no image; keep native text paste

                    var canvas = document.createElement('canvas');
                    canvas.width = img.width;
                    canvas.height = img.height;
                    var ctx = canvas.getContext('2d');
                    var rgba = new Uint8ClampedArray(b64ToBytes(img.data));
                    ctx.putImageData(new ImageData(rgba, img.width, img.height), 0, 0);

                    canvas.toBlob(function(blob) {
                        var file = new File([blob], 'clipboard.png', { type: 'image/png' });
                        var dt = new DataTransfer();
                        dt.items.add(file);

                        var evt;
                        try {
                            evt = new ClipboardEvent('paste', {
                                clipboardData: dt, bubbles: true, cancelable: true
                            });
                        } catch (err) {
                            evt = new Event('paste', { bubbles: true, cancelable: true });
                            Object.defineProperty(evt, 'clipboardData', { value: dt });
                        }
                        e.target.dispatchEvent(evt);
                        e.preventDefault();
                    }, 'image/png');
                })
                .catch(function() {});
        }, true);
    }
})();
"#;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_clipboard_image])
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
