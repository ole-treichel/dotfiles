vim.opt.tabstop = 2
vim.opt.softtabstop = 2
vim.opt.shiftwidth = 2
vim.opt.expandtab = true
vim.opt.list = true
vim.opt.listchars = {
  tab = '┊ ',
  leadmultispace = '┊ ',
  trail = '␣',
  nbsp = '⍽',
}
vim.opt.nu = true
vim.opt.relativenumber = true
vim.opt.signcolumn = 'yes:1'
vim.opt.wrap = false
vim.opt.termguicolors = true
vim.opt.timeoutlen = 300
-- show statusbar only for current file
vim.opt.laststatus = 3
-- bun hot / watch fix: https://github.com/oven-sh/bun/issues/8520#issuecomment-2002325950
vim.opt.backupcopy = 'yes'
vim.filetype.add {
  extension = {
    templ = 'templ',
    mjml = 'html',
    re = 'reason'
  },
}

vim.diagnostic.config{
  virtual_lines = true,
}

--[[ vim.diagnostic.config{
  virtual_text = {
    virt_text_pos = 'eol_right_align',
  },
} ]]

--[[ vim.diagnostic.config({ virtual_text = false }) ]]
