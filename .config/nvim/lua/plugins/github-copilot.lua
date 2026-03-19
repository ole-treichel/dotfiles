return {
  'github/copilot.vim',
  config = function()
    vim.g.copilot_no_tab_map = true
    vim.keymap.set("i", "<C-l>", "copilot#Accept()", { noremap = true, silent = true, expr = true, replace_keycodes = false })
    vim.keymap.set("i", "<C-j>", "copilot#Next()", { expr = true, silent = true })
    vim.keymap.set("i", "<C-k>", "copilot#Previous()", { expr = true, silent = true })
  end
}
