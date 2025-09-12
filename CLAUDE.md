# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is a personal dotfiles repository containing configuration files for various development tools and terminal environments.

## Architecture & Structure

### Core Components

- **Neovim Configuration** (`.config/nvim/`): Lua-based configuration using lazy.nvim plugin manager
  - `lua/config/`: Core configuration modules (init, lazy, remap, set, statusline)
  - `lua/plugins/`: Individual plugin configurations (LSP, Telescope, Treesitter, etc.)
  - Uses lazy.nvim for plugin management with automatic installation

- **Tmux Configuration** (`.tmux.conf`): Terminal multiplexer setup
  - Custom prefix: `Ctrl+Space`
  - Split bindings: `+` (horizontal), `-` (vertical)
  - Integrates with vim-tmux-navigator and tms (tmux sessionizer)
  - Mouse support enabled with vi-mode key bindings

- **TMS Configuration** (`tms/config.toml`): Tmux session manager
  - Default session: "workspace"
  - Searches `/home/ole/workspace` directory (depth 3)
  - Tmux keybinds: `prefix+p` (tms), `prefix+s` (tms switch)

- **Terminal Themes** (`gnome-terminal/`): Rose Pine theme for GNOME Terminal

### Key Configuration Details

- **Neovim**: 2-space indentation, relative line numbers, global statusline, supports templ/mjml/reason filetypes
- **Tmux**: Starts windows/panes at index 1, auto-renumbers windows, uses xclip for clipboard integration
- **Development Focus**: TypeScript/JavaScript, Go (templ), ReasonML, HTML/CSS workflows

## Common Tasks

### Applying Configurations

**GNOME Terminal Theme:**
```bash
cd gnome-terminal && dconf load /org/gnome/terminal/legacy/profiles:/ < rose-pine.dconf
```

**Tmux Plugin Installation:**
- Plugins managed via TPM (Tmux Plugin Manager)
- Install: `prefix + I` in tmux session

**Neovim Setup:**
- Lazy.nvim auto-installs on first run
- Plugins configured in `lua/plugins/` directory

### Key Tmux Bindings
- `Ctrl+Space + p`: Open tms (session switcher)
- `Ctrl+Space + s`: Switch tms sessions
- `Ctrl+Space + +`: Split horizontal
- `Ctrl+Space + -`: Split vertical
- `Ctrl+Space + m`: Maximize pane

### Neovim Leader Key
- Leader: `Space`
- LocalLeader: `\`

## File Organization

Configuration files follow standard XDG structure under `.config/`. The Neovim configuration is modular with clear separation between core settings and plugin-specific configurations.