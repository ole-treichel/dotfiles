--[[ return {
  'ole-treichel/poimandres.nvim',
  lazy = false,
  priority = 1000,
  config = function()
    require('poimandres').setup {
      -- leave this setup function empty for default config
      -- or refer to the configuration section
      -- for configuration options
      dim_nc_background = false, -- dim 'non-current' window backgrounds
      disable_background = false, -- disable background
    }
  end,

  -- optionally set the colorscheme within lazy config
  init = function()
    vim.cmd("colorscheme poimandres")
  end
}
]]

return { 
  "catppuccin/nvim", name = "catppuccin", priority = 1000,
  init = function()
    vim.cmd("colorscheme catppuccin-frappe")
  end
}

