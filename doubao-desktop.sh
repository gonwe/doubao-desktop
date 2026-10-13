#!/bin/sh
# Doubao Desktop - Tauri-based native client wrapper
#
# Sets the WebKitGTK environment the app expects, then hands off to the real
# binary. Every variable below is only exported when the user has not already
# set it, so anything can be overridden from the calling environment.

set -eu

# Avoid blank WebKitGTK rendering from GBM/DMABuf on Wayland sessions.
if [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
    WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
    export WEBKIT_DISABLE_DMABUF_RENDERER
fi

# Fix Wayland protocol error on some compositors.
if [ -z "${GDK_BACKEND:-}" ] && [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
    WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
    export WEBKIT_DISABLE_COMPOSITING_MODE
fi

# Use system FreeType settings for consistent CJK hinting.
FREETYPE_PROPERTIES="${FREETYPE_PROPERTIES:-truetype:interpreter-version=35}"
export FREETYPE_PROPERTIES

# Force WebKit to re-read fontconfig.
WEBKIT_FORCE_FONT_CONFIG="${WEBKIT_FORCE_FONT_CONFIG:-1}"
export WEBKIT_FORCE_FONT_CONFIG

# Resolve the binary next to this script so the wrapper also works from a
# non-/usr prefix and from an unpacked source tree.
self_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

for candidate in "$self_dir/doubao-desktop.bin" /usr/bin/doubao-desktop.bin; do
    if [ -x "$candidate" ]; then
        exec "$candidate" "$@"
    fi
done

if command -v doubao-desktop.bin >/dev/null 2>&1; then
    exec doubao-desktop.bin "$@"
fi

echo "doubao-desktop: cannot locate doubao-desktop.bin" >&2
exit 1
