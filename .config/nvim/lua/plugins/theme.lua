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

--[[ return {
  "catppuccin/nvim", name = "catppuccin", priority = 1000,
  config = function()
    require("catppuccin").setup({
      custom_highlights = function()
        return {
          ["@tag.attribute"] = { style = {} },
        }
      end,
    })
    vim.cmd("colorscheme catppuccin-frappe")
  end
}
]]

return {
  "neanias/everforest-nvim",
  version = false,
  lazy = false,
  priority = 1000, -- make sure to load this before all the other start plugins
  -- Optional; default configuration will be used if setup isn't called.
  config = function()
    require("everforest").setup({
      -- Your config here
    })

    vim.cmd([[colorscheme everforest]])
  end,
}

