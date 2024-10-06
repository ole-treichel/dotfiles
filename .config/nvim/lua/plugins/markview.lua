return {
  'OXY2DEV/markview.nvim',
  lazy = false, -- Recommended
  -- ft = "markdown" -- If you decide to lazy-load anyway

  dependencies = {
    'nvim-treesitter/nvim-treesitter',
    'nvim-tree/nvim-web-devicons',
  },

  config = function()
    require('markview').setup {
      initial_state = false,
    }

    vim.keymap.set('n', '<leader>md', '<cmd>Markview toggle<CR>', {})
  end,
}
