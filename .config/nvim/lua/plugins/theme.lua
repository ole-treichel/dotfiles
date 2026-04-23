-- Follow the system color-scheme (org.gnome.desktop.interface color-scheme).
-- The `theme` CLI sets this value; the ColorScheme autocmd from everforest
-- handles live reloads when `:set background=light|dark` is pushed via
-- `nvim --server ... --remote-send`.
local function system_background()
  local ok, handle = pcall(io.popen, "gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null")
  if not ok or not handle then return "dark" end
  local out = handle:read("*a") or ""
  handle:close()
  if out:find("prefer%-light") then return "light" end
  return "dark"
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
