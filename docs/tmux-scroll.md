# tmux wheel scroll speed

## Problem

Scrolling Claude Code output (tmux copy mode, not the app's own scroll) jumped too far per wheel event. Lines just above the viewport were skipped.

## Cause

Two multipliers stack:

- Ghostty `mouse-scroll-multiplier` default: `discrete:3` (3 wheel reports per mouse notch), `precision:1` (1 report per cell height of touchpad travel).
- tmux 3.7 default copy-mode wheel binding: `send-keys -X -N 5 scroll-up`, i.e. 5 lines per report.

Result: 15 lines per mouse notch, 5-line jumps on touchpad.

## Decision

Override `WheelUpPane` / `WheelDownPane` in `copy-mode-vi` (`.tmux.conf`) with `-N 1`.

Ghostty (`.config/ghostty.config`): `mouse-scroll-multiplier = precision:0.5,discrete:1`. Mouse notch = 1 line, touchpad = 1 line per 2 cells of travel. Further reduction: lower `precision` (e.g. `0.3`); `discrete` below 1 makes some notches scroll nothing.

Only `copy-mode-vi` is bound because `mode-keys vi` is set; the emacs `copy-mode` table is never active.

## Non-goals

- Pixel-smooth scrolling. tmux redraws in whole cells; Ghostty's smooth scroll applies only to its own scrollback, which tmux bypasses. 1 line per step is the finest granularity available.
- Velocity-dependent / accelerated scrolling. tmux has no event timing in bindings.
