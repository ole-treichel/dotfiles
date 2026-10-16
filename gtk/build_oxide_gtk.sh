#!/usr/bin/env bash
# Build and install the Oxide GTK3 / GTK4 (libadwaita) / GNOME Shell theme as
# ~/.themes/Oxide-Dark and ~/.themes/Oxide-Light.
#
# Source: Everforest-GTK-Theme (Colloid-based) at a pinned commit, with:
#   _color-palette-oxide.scss  oxide palette in upstream's variable names
#   oxide.patch                semantic remap (surfaces, selection, accents)
#   render_png_assets.py       GTK3 PNG assets via librsvg (no inkscape)
#
# Usage: build_oxide_gtk.sh [existing upstream checkout]
#   DEST_DIR=<dir> overrides the install location (default ~/.themes).
set -euo pipefail

UPSTREAM_URL="https://github.com/Fausto-Korpsvart/Everforest-GTK-Theme.git"
UPSTREAM_COMMIT="9b8be4d6648ae9eaae3dd550105081f8c9054825"
THEME_NAME="Oxide"
DEST_DIR="${DEST_DIR:-$HOME/.themes}"
SASSC_OPT=(-M -t expanded)

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

for cmd in git sassc python3; do
	command -v "$cmd" >/dev/null || { echo "missing: $cmd" >&2; exit 1; }
done

# --- Fetch upstream at the pinned commit -------------------------------------
UPSTREAM="$WORK/upstream"
mkdir -p "$UPSTREAM"
if [[ $# -ge 1 ]]; then
	git -C "$1" archive "$UPSTREAM_COMMIT" | tar -x -C "$UPSTREAM"
else
	git -C "$UPSTREAM" init -q
	git -C "$UPSTREAM" remote add origin "$UPSTREAM_URL"
	git -C "$UPSTREAM" fetch -q --depth 1 origin "$UPSTREAM_COMMIT"
	git -C "$UPSTREAM" checkout -q FETCH_HEAD
fi

SRC="$UPSTREAM/themes/src"

# --- Apply oxide palette and remap ---------------------------------------------
(cd "$UPSTREAM" && git apply "$HERE/oxide.patch")
cp "$HERE/_color-palette-oxide.scss" "$SRC/sass/"

sed -e "/@import/s/color-palette-default/color-palette-oxide/" \
	-e "/\$colorscheme:/s/default/oxide/" \
	"$SRC/sass/_tweaks.scss" >"$SRC/sass/_tweaks-temp.scss"

# GNOME Shell widget set: newest upstream ships is 48-0 (matches install.sh).
GS_VERSION="48-0"
if command -v gnome-shell >/dev/null; then
	gs_major="$(gnome-shell --version | cut -d ' ' -f 3 | cut -d . -f 1)"
	if ((gs_major < 48)); then
		for v in 47 46 44 42 40; do
			if ((gs_major >= v)); then GS_VERSION="$v-0"; break; fi
		done
	fi
fi
sed "/widgets/s/40-0/${GS_VERSION}/" \
	"$SRC/sass/gnome-shell/_common.scss" >"$SRC/sass/gnome-shell/_common-temp.scss"

# --- Recolor color-baked assets --------------------------------------------------
# Upstream bakes Everforest (and leftover Catppuccin) colors into the GTK3 PNG
# source and the Shell switch/checkbox SVGs. GTK4 assets are recolored at
# runtime (-gtk-recolor) and need no change.
recolor() {
	sed -i \
		-e 's/#3a94c5/#333333/gI' -e 's/#009688/#333333/gI' \
		-e 's/#7fbbb3/#cecece/gI' -e 's/#4db6ac/#cecece/gI' \
		-e 's/#fffbef/#f5f5f5/gI' -e 's/#eff1f5/#f5f5f5/gI' \
		-e 's/#e8e5d5/#e4e4e4/gI' -e 's/#dce0e8/#d4d4d4/gI' \
		-e 's/#272e33/#161616/gI' -e 's/#1e1e2e/#161616/gI' \
		-e 's/#293136/#222222/gI' \
		-e 's/#374145/#2e2e2e/gI' -e 's/#35364a/#2e2e2e/gI' \
		"$@"
}
# GTK3: #1e2326 is the dark base layer (mantle). Shell: it is the checkbox
# mark and switch knob on the dark accent, so it becomes base.
recolor -e 's/#1e2326/#121212/gI' "$SRC/assets/gtk/assets.svg"
recolor -e 's/#1e2326/#161616/gI' "$SRC/assets/gnome-shell/theme/"*.svg

GTK3_PNG="$WORK/gtk3-png"
python3 "$HERE/render_png_assets.py" \
	"$SRC/assets/gtk/assets.svg" "$SRC/assets/gtk/assets.txt" "$GTK3_PNG"

# --- Build and install each variant ----------------------------------------------
install_variant() {
	local color="$1" # Dark | Light
	local name="${THEME_NAME}-${color}"
	local out="$WORK/out/$name"
	local shell_assets="assets"
	[[ "$color" == "Dark" ]] && shell_assets="assets-Dark"

	mkdir -p "$out"/{gnome-shell,gtk-3.0,gtk-4.0}

	cat >"$out/index.theme" <<-EOF
		[Desktop Entry]
		Type=X-GNOME-Metatheme
		Name=${name}
		Comment=Oxide (oxidescheme) built from Everforest-GTK-Theme ${UPSTREAM_COMMIT:0:7}
		Encoding=UTF-8

		[X-GNOME-Metatheme]
		GtkTheme=${name}
		IconTheme=Adwaita
		CursorTheme=Adwaita
		ButtonLayout=close,minimize,maximize:menu
	EOF

	# GNOME Shell
	sassc "${SASSC_OPT[@]}" "$SRC/main/gnome-shell/gnome-shell-${color}.scss" "$out/gnome-shell/gnome-shell.css"
	cp "$SRC/main/gnome-shell/pad-osd.css" "$out/gnome-shell/"
	cp -r "$SRC/assets/gnome-shell/common-assets" "$out/gnome-shell/assets"
	cp "$SRC/assets/gnome-shell/${shell_assets}/"*.svg "$out/gnome-shell/assets/"
	cp "$SRC/assets/gnome-shell/theme/"*.svg "$out/gnome-shell/assets/"
	ln -s assets/no-events.svg "$out/gnome-shell/no-events.svg"
	ln -s assets/process-working.svg "$out/gnome-shell/process-working.svg"
	ln -s assets/no-notifications.svg "$out/gnome-shell/no-notifications.svg"

	# GTK3
	cp -r "$GTK3_PNG" "$out/gtk-3.0/assets"
	cp -r "$SRC/assets/gtk/scalable" "$out/gtk-3.0/assets/scalable"
	sassc "${SASSC_OPT[@]}" "$SRC/main/gtk-3.0/gtk-${color}.scss" "$out/gtk-3.0/gtk.css"
	sassc "${SASSC_OPT[@]}" "$SRC/main/gtk-3.0/gtk-Dark.scss" "$out/gtk-3.0/gtk-dark.css"

	# GTK4 / libadwaita
	cp -r "$SRC/assets/gtk/scalable" "$out/gtk-4.0/assets"
	sassc "${SASSC_OPT[@]}" "$SRC/main/gtk-4.0/gtk-${color}.scss" "$out/gtk-4.0/gtk.css"
	sassc "${SASSC_OPT[@]}" "$SRC/main/gtk-4.0/gtk-Dark.scss" "$out/gtk-4.0/gtk-dark.css"

	mkdir -p "$DEST_DIR"
	rm -rf "${DEST_DIR:?}/$name"
	cp -a "$out" "$DEST_DIR/$name"
	echo "installed $DEST_DIR/$name"
}

install_variant Dark
install_variant Light
