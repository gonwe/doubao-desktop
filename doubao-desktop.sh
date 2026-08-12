#!/bin/sh
# Doubao Desktop - Tauri-based native client wrapper
# Handles Wayland/GBM compatibility on some GPU/driver combinations

# Fix Wayland protocol error on some compositors
# See: https://github.com/tauri-apps/tauri/issues/9304
if [ -z "$GDK_BACKEND" ] && [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
fi

exec /usr/bin/doubao-desktop.bin "$@"
