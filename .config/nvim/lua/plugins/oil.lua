return {
  'stevearc/oil.nvim',
  opts = {},
  -- Optional dependencies
  dependencies = { 'nvim-tree/nvim-web-devicons' },

  config = function()
    require('oil').setup {
      view_options = {
        show_hidden = true,
      },
    }

    vim.keymap.set('n', '<leader>ls', '<CMD>Oil<CR>')
  end,
}
