return {
  'nvim-telescope/telescope.nvim',

  tag = '0.1.5',

  dependencies = {
    'nvim-lua/plenary.nvim',
  },

  config = function()
    local previewers = require("telescope.previewers")

    local _bad = { ".*%.min.js", ".*%.min.css", } -- Put all filetypes that slow you down in this array
    local bad_files = function(filepath)
      for _, v in ipairs(_bad) do
        if filepath:match(v) then
          return false
        end
      end

      return true
    end

    local new_maker = function(filepath, bufnr, opts)
      opts = opts or {}
      if opts.use_ft_detect == nil then opts.use_ft_detect = true end
      opts.use_ft_detect = opts.use_ft_detect == false and false or bad_files(filepath)
      previewers.buffer_previewer_maker(filepath, bufnr, opts)
    end

    require('telescope').setup({
      defaults = {
        buffer_previewer_maker = new_maker,
      }
    })

    local builtin = require 'telescope.builtin'

    vim.keymap.set('n', '<leader>fa', builtin.find_files, {})
    vim.keymap.set('n', '<leader>ff', builtin.git_files, {})
    vim.keymap.set('n', '<leader>ffg', function()
      builtin.grep_string { search = vim.fn.input 'Grep > ' }
    end)
  end,
}
