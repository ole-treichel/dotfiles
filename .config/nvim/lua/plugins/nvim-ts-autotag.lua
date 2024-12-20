return {
  'windwp/nvim-ts-autotag',

  config = function()
    require('nvim-ts-autotag').setup {
      filetypes = { 'html', 'go', 'javascriptreact', 'jsx' },
      aliases = { go = 'html' },
    }
  end,
}
