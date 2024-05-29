return {
  'stevearc/oil.nvim',
  opts = {},
  -- Optional dependencies
  dependencies = { 'nvim-tree/nvim-web-devicons' },

  config = function()
    require('oil').setup {
      use_default_keymaps = false,

      experimental_watch_for_changes = false,

      keymaps = {
        ['<CR>'] = 'actions.select',
        ['-'] = 'actions.parent',
        ['<C-r>'] = 'actions.refresh',
      },

      view_options = {
        show_hidden = true,
      },
    }

    vim.keymap.set('n', '<leader>ls', '<CMD>Oil<CR>')
  end,
}
