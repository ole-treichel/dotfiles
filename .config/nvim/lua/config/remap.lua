vim.g.mapleader = ' '

-- moving line(s) with "J" / "K"
vim.keymap.set('v', 'K', ':m \'>-2<CR>gv=gv')
vim.keymap.set('v', 'J', ':m \'>+1<CR>gv=gv')

-- keep cursor in middle when halfpage scrolling with <C-d> / <C-u>
vim.keymap.set('n', '<C-d>', '<C-d>zz')
vim.keymap.set('n', '<C-u>', '<C-u>zz')

-- keep cursor in moddle when searching
vim.keymap.set('n', 'n', 'nzzzv')
vim.keymap.set('n', 'N', 'Nzzzv')

-- disable recording
vim.keymap.set('n', 'q', '<Nop>')
vim.keymap.set('n', 'Q', '<nop>')

-- global clipboard shortcut
vim.keymap.set('v', '<leader>y', '"+y')
vim.keymap.set('v', '<leader>p', '"+p')
vim.keymap.set('n', '<leader>p', '"+p')

-- window management
vim.keymap.set('n', '<leader>sv', '<C-w>v') -- split window vertically
vim.keymap.set('n', '<leader>sh', '<C-w>s') -- split window horizontally
vim.keymap.set('n', '<leader>se', '<C-w>=') -- make split windows equal width & height
vim.keymap.set('n', '<leader>sx', ':close<CR>') -- close current split window

-- for vscode noobs
vim.keymap.set('n', '<Left>', '<Nop>')
vim.keymap.set('n', '<Right>', '<Nop>')
vim.keymap.set('n', '<Up>', '<Nop>')
vim.keymap.set('n', '<Down>', '<Nop>')

vim.keymap.set('i', '<Left>', '<Nop>')
vim.keymap.set('i', '<Right>', '<Nop>')
vim.keymap.set('i', '<Up>', '<Nop>')
vim.keymap.set('i', '<Down>', '<Nop>')

-- to clear hightlights
vim.keymap.set('n', '<leader>nh', ':nohl<CR>')

-- fix netrw blocking <C-l> to move to right split
vim.api.nvim_create_autocmd('filetype', {
  pattern = 'netrw',
  desc = 'Better mappings for netrw',
  callback = function()
    local bind = function(lhs, rhs)
      vim.keymap.set('n', lhs, rhs, { remap = true, buffer = true })
    end

    bind('<C-l>', ':TmuxNavigateRight<cr>')
  end,
})

-- prevent overwriting clipboard when pasting
vim.keymap.set('v', 'd', '"_d"')
vim.keymap.set('v', 'dd', '"_dd"')
vim.api.nvim_set_keymap('v', 'p', 'P', { noremap = true })
vim.keymap.set('n', 'd', '"_d"')
vim.keymap.set('n', 'diw', '"_diw"')
vim.keymap.set('n', 'dd', '"_dd"')
vim.api.nvim_set_keymap('v', 'p', 'P', { noremap = true })

-- show hightlight for search but clear when hitting escape
vim.opt.hlsearch = true
vim.keymap.set('n', '<Esc>', '<cmd>nohlsearch<CR>')

-- wrap word in qoutes
vim.keymap.set('n', '"', 'ciw"<Esc>pa"<Esc>')
vim.keymap.set('n', '\'', 'ciw\'<Esc>pa\'<Esc>')

-- wrap word in parantheses
vim.keymap.set('n', '(', 'ciw(<Esc>pa)<Esc>')
vim.keymap.set('n', '"', 'ciw"<Esc>pa"<Esc>')
