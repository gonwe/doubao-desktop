#!/bin/sh
# Doubao Desktop - Tauri-based native client wrapper

# Fix Wayland protocol error on some compositors
if [ -z "$GDK_BACKEND" ] && [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
fi

# Improve CJK font rendering in WebKitGTK
# Use system FreeType settings for consistent CJK hinting
export FREETYPE_PROPERTIES="truetype:interpreter-version=35"
# Force WebKit to re-read fontconfig
export WEBKIT_FORCE_FONT_CONFIG=1

exec /usr/bin/doubao-desktop.bin "$@"
