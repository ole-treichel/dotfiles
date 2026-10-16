#!/usr/bin/env python3
"""Render the GTK3 PNG assets: one PNG per element id, at 1x and 2x.

Replaces upstream's render-assets.sh, which needs inkscape or rendersvg.
Uses librsvg through GObject introspection (present on any GNOME system).

Usage: render_png_assets.py <assets.svg> <assets.txt> <output dir>
"""
import sys
from pathlib import Path

import gi

gi.require_version("Rsvg", "2.0")
import cairo  # noqa: E402
from gi.repository import Rsvg  # noqa: E402


def render(handle: Rsvg.Handle, element_id: str, scale: int, out: Path) -> None:
    ok, ink, _logical = handle.get_geometry_for_element(f"#{element_id}")
    if not ok:
        raise SystemExit(f"element not found: {element_id}")
    width = max(1, round(ink.width * scale))
    height = max(1, round(ink.height * scale))
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, width, height)
    ctx = cairo.Context(surface)
    viewport = Rsvg.Rectangle()
    viewport.x, viewport.y = 0, 0
    viewport.width, viewport.height = ink.width * scale, ink.height * scale
    handle.render_element(ctx, f"#{element_id}", viewport)
    surface.write_to_png(str(out))


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    svg, index, out_dir = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    out_dir.mkdir(parents=True, exist_ok=True)
    handle = Rsvg.Handle.new_from_file(str(svg))
    for element_id in index.read_text().split():
        render(handle, element_id, 1, out_dir / f"{element_id}.png")
        render(handle, element_id, 2, out_dir / f"{element_id}@2.png")


if __name__ == "__main__":
    main()
