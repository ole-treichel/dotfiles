-- Follow the system color-scheme. The `theme` CLI sets the system value;
-- live reloads happen when `:set background=light|dark` is pushed via
-- `nvim --server ... --remote-send`.
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
    local ok, handle = pcall(io.popen, "gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null")
    if not ok or not handle then return "dark" end
    local out = handle:read("*a") or ""
    handle:close()
    if out:find("prefer%-light") then return "light" end
    return "dark"
  end
end

return {
  "neanias/everforest-nvim",
  version = false,
  lazy = false,
  priority = 1000,
  config = function()
    vim.opt.background = system_background()
    require("everforest").setup({
      background = "soft",
    })
    vim.cmd([[colorscheme everforest]])
  end,
}
