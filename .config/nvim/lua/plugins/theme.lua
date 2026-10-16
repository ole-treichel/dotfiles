-- Follow the system color-scheme. The `theme` CLI sets the system value;
-- live reloads happen when `:set background=light|dark` is pushed via
-- `nvim --server ... --remote-send`. Changing 'background' reloads the
-- active colorscheme, which re-runs `on_colors` below.
local function system_background()
  local sysname = vim.loop.os_uname().sysname
  if sysname == "Darwin" then
    local ok, handle = pcall(io.popen, "defaults read -g AppleInterfaceStyle 2>/dev/null")
    if not ok or not handle then return "light" end
    local out = handle:read("*a") or ""
    handle:close()
    if out:find("Dark") then return "dark" end
    return "light"
  else
    local ok, handle = pcall(io.popen, "env -u GSETTINGS_BACKEND gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null")
    if not ok or not handle then return "dark" end
    local out = handle:read("*a") or ""
    handle:close()
    if out:find("prefer%-light") then return "light" end
    return "dark"
  end
end

-- oxide is dark-only upstream. Light palette derived in the dotfiles repo,
-- see oxide/colors.txt and docs/oxide-theme.md.
local light = {
  mantle = "#eeeeee",
  base = "#f5f5f5",
  surface0 = "#e4e4e4",
  surface1 = "#d4d4d4",
  surface2 = "#c4c4c4",

  bright_text = "#1b1b1b",
  text = "#333333",
  subtext0 = "#4d4d4d",
  subtext1 = "#777777",
  subtext2 = "#a4a4a4",

  red = "#af3c3a",
  orange = "#a74a00",
  yellow = "#8a6200",
  lime = "#687100",
  green = "#1b7e2a",
  teal = "#008274",
  sky = "#007b9e",
  blue = "#006eb8",
  purple = "#6358ba",
  pink = "#924598",

  bright_red = "#972527",
  bright_orange = "#903500",
  bright_yellow = "#754d00",
  bright_lime = "#545d00",
  bright_green = "#006911",
  bright_teal = "#006d60",
  bright_sky = "#006688",
  bright_blue = "#0058a1",
  bright_purple = "#5043a3",
  bright_pink = "#7c2f82",

  diff_delete = "#f8d4d1",
  diff_add = "#cee6ce",
  diff_change = "#c9e2f7",
}

return {
  "oxidescheme/nvim",
  name = "oxide",
  version = false,
  lazy = false,
  priority = 1000,
  config = function()
    vim.opt.background = system_background()
    require("oxide").setup({
      on_colors = function(colors)
        if vim.o.background == "light" then
          return vim.tbl_extend("force", colors, light)
        end
        return colors
      end,
    })
    vim.cmd.colorscheme("oxide")
  end,
}
