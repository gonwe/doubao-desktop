// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::PathBuf, process::Command};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

const DEFAULT_URL: &str = "https://www.doubao.com/chat/";
const DEFAULT_HIDE_LABELS: &str = "下载豆包电脑版";

#[derive(serde::Serialize)]
struct ClipboardImage {
    width: usize,
    height: usize,
    /// RGBA8 pixel data, base64-encoded.
    data: String,
}

#[derive(serde::Serialize)]
struct ClipboardFile {
    name: String,
    mime_type: String,
    /// File data, base64-encoded for transfer over Tauri IPC.
    data: String,
}

#[derive(serde::Serialize)]
struct ClipboardPayload {
    image: Option<ClipboardImage>,
    files: Vec<ClipboardFile>,
}

/// arboard's URI-list parser leaves the CR from CRLF line endings attached to
/// the decoded path. Use the corrected path only when it resolves to a file.
fn local_clipboard_file(path: PathBuf) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path);
    }

    let corrected = PathBuf::from(path.to_str()?.strip_suffix('\r')?);
    corrected.is_file().then_some(corrected)
}

/// Read image or copied local files from the system clipboard.
///
/// WebKitGTK does not reliably expose image/file data through DOM paste events,
/// so the page asks for the native clipboard payload during a user paste.
#[tauri::command]
fn get_clipboard_payload() -> Result<ClipboardPayload, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    let files = clipboard
        .get()
        .file_list()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|path| {
            let path = local_clipboard_file(path)?;
            let bytes = fs::read(&path).ok()?;
            let name = path.file_name()?.to_string_lossy().into_owned();
            let mime_type = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();

            Some(ClipboardFile {
                name,
                mime_type,
                data: STANDARD.encode(bytes),
            })
        })
        .collect();

    let image = clipboard.get_image().ok().map(|img| ClipboardImage {
        width: img.width,
        height: img.height,
        data: STANDARD.encode(img.bytes.as_ref()),
    });

    Ok(ClipboardPayload { image, files })
}

/// Runtime configuration, all overridable through the environment.
struct Config {
    url: String,
    /// `None` keeps WebKit's native UA. Overriding it is opt-in because
    /// spoofing a different platform changes the fingerprint the site sees.
    user_agent: Option<String>,
    hide_labels: Vec<String>,
}

impl Config {
    fn from_env() -> Self {
        let url = env_non_empty("DOUBAO_URL").unwrap_or_else(|| DEFAULT_URL.to_string());
        let user_agent = env_non_empty("DOUBAO_USER_AGENT");
        let hide_labels = env_non_empty("DOUBAO_HIDE_BUTTONS")
            .unwrap_or_else(|| DEFAULT_HIDE_LABELS.to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cfg = Self {
            url,
            user_agent,
            hide_labels,
        };

        if std::env::var_os("DOUBAO_DEBUG").is_some() {
            eprintln!(
                "doubao-desktop: url={} user_agent={} hide_labels={:?}",
                cfg.url,
                cfg.user_agent.as_deref().unwrap_or("<webkit default>"),
                cfg.hide_labels,
            );
        }

        cfg
    }
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Only doubao.com stays inside the app window; anything else (docs, terms,
/// third-party links) is handed to the system browser so the wrapper never
/// wanders away from the chat UI and loses the user's session.
fn is_doubao(host: Option<&str>) -> bool {
    let Some(h) = host else { return false };
    let h = h.to_ascii_lowercase();
    // The leading dot in the suffix check rejects lookalikes like "evildoubao.com".
    h == "doubao.com" || h.ends_with(".doubao.com")
}

const INIT_SCRIPT: &str = r#"
(function() {
    var HIDE_LABELS = __HIDE_LABELS__;

    // CJK font rendering tweaks for WebKitGTK.
    // NOTE: this script runs at document start (UserScriptInjectionTime::Start),
    // so document.head may not exist yet — guard against it.
    var s = document.createElement('style');
    s.textContent = '*{font-family:"Douyin Sans","抖音美好体",'
        +'"PingFang SC","Noto Sans CJK SC","Source Han Sans CN",'
        +'"Microsoft YaHei","WenQuanYi Micro Hei",'
        +'sans-serif!important;-webkit-font-smoothing:antialiased!important}';
    (document.head || document.documentElement).appendChild(s);

    // --- Hide "download the desktop client" prompts ---
    // Uses a MutationObserver rather than a polling loop: doubao re-renders
    // large parts of the DOM on every SPA navigation, so scanning every
    // <button> once a second is a constant CPU cost for the whole session.
    // The observer only runs when nodes are actually added.
    function labelMatches(el) {
        if (!el || el.nodeType !== 1 || el.tagName !== 'BUTTON') return false;
        var t = (el.textContent || '').trim();
        for (var i = 0; i < HIDE_LABELS.length; i++) {
            if (t === HIDE_LABELS[i]) return true;
        }
        return false;
    }

    function hideIn(root) {
        if (!root) return;
        if (labelMatches(root)) root.style.display = 'none';
        if (!root.querySelectorAll) return;
        var buttons = root.querySelectorAll('button');
        for (var i = 0; i < buttons.length; i++) {
            if (labelMatches(buttons[i])) buttons[i].style.display = 'none';
        }
    }

    function onMutations(muts) {
        for (var i = 0; i < muts.length; i++) {
            var added = muts[i].addedNodes;
            for (var j = 0; j < added.length; j++) hideIn(added[j]);
        }
    }

    function startHideObserver() {
        if (!HIDE_LABELS.length || !document.body) return;
        hideIn(document.body);
        new MutationObserver(onMutations).observe(document.body, {
            childList: true,
            subtree: true
        });
    }

    // --- Image and file paste workaround for WebKitGTK ---
    // WebKitGTK may omit native image/file data from DOM paste events. Read the
    // user's clipboard during paste and re-inject it as a synthetic paste.
    function installPasteFix() {
        if (!window.__TAURI_INTERNALS__ || !window.__TAURI_INTERNALS__.invoke) return;

        function b64ToBytes(b64) {
            var bin = atob(b64);
            var bytes = new Uint8Array(bin.length);
            for (var i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
            return bytes;
        }

        function dispatchPaste(target, pastedFiles) {
            if (!pastedFiles.length) return;

            var dt = new DataTransfer();
            for (var i = 0; i < pastedFiles.length; i++) dt.items.add(pastedFiles[i]);

            var evt;
            try {
                evt = new ClipboardEvent('paste', {
                    clipboardData: dt, bubbles: true, cancelable: true
                });
            } catch (err) {
                evt = new Event('paste', { bubbles: true, cancelable: true });
                Object.defineProperty(evt, 'clipboardData', { value: dt });
            }
            target.dispatchEvent(evt);
        }

        function inputAcceptsFile(input, file) {
            var accept = (input.accept || '').split(',').map(function(rule) {
                return rule.trim().toLowerCase();
            }).filter(Boolean);
            if (!accept.length) return true;

            var name = file.name.toLowerCase();
            var type = (file.type || '').toLowerCase();
            return accept.some(function(rule) {
                if (rule.charAt(0) === '.') return name.endsWith(rule);
                if (rule.endsWith('/*')) return type.indexOf(rule.slice(0, -1)) === 0;
                return type === rule;
            });
        }

        // Doubao's document uploader may listen on its file input rather than
        // accept non-image files from a paste event. Feed the copied files
        // through the same input/change path used by its attachment button.
        function injectIntoFileInput(pastedFiles) {
            var inputs = document.querySelectorAll('input[type="file"]');
            var candidates = [];
            for (var i = 0; i < inputs.length; i++) {
                var input = inputs[i];
                if (input.disabled) continue;
                var acceptsAll = pastedFiles.every(function(file) {
                    return inputAcceptsFile(input, file);
                });
                if (acceptsAll) candidates.push(input);
            }
            candidates.sort(function(a, b) {
                return Number(b.multiple) - Number(a.multiple);
            });

            for (var i = 0; i < candidates.length; i++) {
                var transfer = new DataTransfer();
                for (var j = 0; j < pastedFiles.length; j++) {
                    transfer.items.add(pastedFiles[j]);
                }
                try {
                    candidates[i].files = transfer.files;
                    candidates[i].dispatchEvent(new Event('change', { bubbles: true }));
                    return true;
                } catch (err) {}
            }
            return false;
        }

        document.addEventListener('paste', function(e) {
            var data = e.clipboardData;
            var items = data && data.items;
            var files = data && data.files;

            // Native paste works when WebKit already supplied the payload.
            if (files && files.length) return;
            if (items) {
                for (var i = 0; i < items.length; i++) {
                    if (items[i].kind === 'file' && items[i].getAsFile()) return;
                    if (items[i].type && items[i].type.indexOf('image') === 0
                            && items[i].getAsFile()) return;
                }
            }

            var types = data && data.types;
            var uriList = types && Array.prototype.indexOf.call(types, 'text/uri-list') !== -1;
            // File managers expose copied paths as URI lists. Prevent inserting
            // the raw file:// text while the native files are being read.
            if (uriList) e.preventDefault();

            var target = e.target && e.target.dispatchEvent
                ? e.target : (document.activeElement || document);

            window.__TAURI_INTERNALS__.invoke('get_clipboard_payload')
                .then(function(payload) {
                    if (!payload) return;

                    var pastedFiles = [];
                    if (payload.files && payload.files.length) {
                        for (var i = 0; i < payload.files.length; i++) {
                            var nativeFile = payload.files[i];
                            pastedFiles.push(new File(
                                [b64ToBytes(nativeFile.data)],
                                nativeFile.name,
                                { type: nativeFile.mime_type || 'application/octet-stream' }
                            ));
                        }
                        if (injectIntoFileInput(pastedFiles)) return;
                        dispatchPaste(target, pastedFiles);
                        return;
                    }

                    if (!payload.image) return;

                    var img = payload.image;
                    var canvas = document.createElement('canvas');
                    canvas.width = img.width;
                    canvas.height = img.height;
                    var ctx = canvas.getContext('2d');
                    var rgba = new Uint8ClampedArray(b64ToBytes(img.data));
                    ctx.putImageData(new ImageData(rgba, img.width, img.height), 0, 0);
                    canvas.toBlob(function(blob) {
                        if (!blob) return;
                        dispatchPaste(target, [new File([blob], 'clipboard.png', {
                            type: 'image/png'
                        })]);
                    }, 'image/png');
                })
                .catch(function(err) {
                    console.error('doubao-desktop: clipboard paste bridge failed', err);
                });
        }, true);
    }

    function boot() {
        startHideObserver();
        installPasteFix();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', boot);
    } else {
        boot();
    }
})();
"#;

fn main() {
    let cfg = Config::from_env();
    let init_script = INIT_SCRIPT.replace(
        "__HIDE_LABELS__",
        &serde_json::to_string(&cfg.hide_labels).unwrap_or_else(|_| "[]".into()),
    );

    let url: tauri::Url = cfg.url.parse().expect("DOUBAO_URL is not a valid URL");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // A second launch (taskbar click, .desktop re-run) should surface the
            // existing window instead of starting a second WebKit process.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![get_clipboard_payload])
        .setup(move |app| {
            let mut window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url));

            if let Some(ua) = &cfg.user_agent {
                window = window.user_agent(ua);
            }

            window
                .title("豆包")
                .inner_size(1200.0, 800.0)
                .min_inner_size(600.0, 400.0)
                .initialization_script(&init_script)
                .on_navigation(|url| {
                    if is_doubao(url.host_str()) {
                        true
                    } else {
                        let _ = Command::new("xdg-open").arg(url.as_str()).spawn();
                        false
                    }
                })
                .build()?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while initializing doubao-desktop")
        .run(|_app, _event| {});
}
#[cfg(test)]
mod tests {
    use super::{is_doubao, local_clipboard_file};
    use std::path::PathBuf;

    #[test]
    fn keeps_doubao_and_subdomains_in_window() {
        for host in [
            "doubao.com",
            "www.doubao.com",
            "accounts.doubao.com",
            "DOUBAO.COM",
            "Www.Doubao.Com",
        ] {
            assert!(is_doubao(Some(host)), "{host} should stay in-window");
        }
    }

    #[test]
    fn hands_everything_else_to_the_system_browser() {
        for host in [
            // Lookalikes that must not be treated as doubao.
            "evildoubao.com",
            "doubao.com.evil.net",
            "notdoubao.com",
            "",
        ] {
            assert!(!is_doubao(Some(host)), "{host} should be external");
        }
        assert!(!is_doubao(None));
    }

    #[test]
    fn repairs_crlf_terminated_clipboard_file_path() {
        let executable = std::env::current_exe().expect("test executable path");
        let mut uri_path = executable.to_string_lossy().into_owned();
        uri_path.push('\r');

        assert_eq!(
            local_clipboard_file(PathBuf::from(uri_path)),
            Some(executable)
        );
    }
}
