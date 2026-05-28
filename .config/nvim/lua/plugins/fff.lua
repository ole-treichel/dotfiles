return {
  'dmtrKovalenko/fff.nvim',
  build = function()
    -- this will download prebuild binary or try to use existing rustup toolchain to build from source
    -- (if you are using lazy you can use gb for rebuilding a plugin if needed)
    require("fff.download").download_or_build_binary()
  end,
  -- if you are using nixos
  -- build = "nix run .#release",
  opts = { -- (optional)
    debug = {
      enabled = true,     -- we expect your collaboration at least during the beta
      show_scores = true, -- to help us optimize the scoring system, feel free to share your scores!
    },
  },
  config = function(_, opts)
    require('fff').setup(opts)

    -- fff.nvim opens picked files in whatever window Neovim focuses after
    -- its floats close (usually the leftmost), not the window the picker
    -- was invoked from. Fully replace M.select: capture the picked path,
    -- close the picker, switch to the origin window, then :edit.
    local picker_ui = require('fff.picker_ui')
    vim.notify('fff.nvim window-origin patch loaded', vim.log.levels.INFO)

    picker_ui.select = function(action)
      if not picker_ui.state.active then return end
      local items = picker_ui.state.filtered_items
      local cursor = picker_ui.state.cursor
      if not items or #items == 0 or cursor > #items then return end
      local item = items[cursor]
      if not item then return end

      action = action or 'edit'
      local rel = vim.fn.fnamemodify(item.path or item.relative_path, ':.')

      -- grep mode: capture line/col before we tear state down
      local location = picker_ui.state.location
      local mode = picker_ui.state.mode
      local suggestion_source = picker_ui.state.suggestion_source
      if (mode == 'grep' or suggestion_source == 'grep') and item.line_number and item.line_number > 0 then
        location = { line = item.line_number }
        if item.col and item.col > 0 then location.col = item.col + 1 end
      end

      local origin = vim.g._fff_origin_win
      vim.g._fff_origin_win = nil

      vim.cmd('stopinsert')
      picker_ui.close()

      if origin and vim.api.nvim_win_is_valid(origin) then
        pcall(vim.api.nvim_set_current_win, origin)
      end

      local esc = vim.fn.fnameescape(rel)
      if action == 'split' then
        vim.cmd('split ' .. esc)
      elseif action == 'vsplit' then
        vim.cmd('vsplit ' .. esc)
      elseif action == 'tab' then
        vim.cmd('tabedit ' .. esc)
      else
        vim.cmd('edit ' .. esc)
      end

      if location then
        vim.schedule(function()
          pcall(vim.api.nvim_win_set_cursor, 0, { location.line, (location.col or 1) - 1 })
        end)
      end
    end
  end,
  -- No need to lazy-load with lazy.nvim.
  -- This plugin initializes itself lazily.
  lazy = false,
  keys = {
    {
      "ff", -- try it if you didn't it is a banger keybinding for a picker
      function()
        vim.g._fff_origin_win = vim.api.nvim_get_current_win()
        require('fff').find_files()
      end,
      desc = 'FFFind files',
    },
    {
      "ffg",
      function()
        vim.g._fff_origin_win = vim.api.nvim_get_current_win()
        require('fff').live_grep()
      end,
      desc = 'LiFFFe grep',
    },
    {
      "fz",
      function()
        vim.g._fff_origin_win = vim.api.nvim_get_current_win()
        require('fff').live_grep({
          grep = {
            modes = { 'fuzzy', 'plain' }
          }
        })
      end,
      desc = 'Live fffuzy grep',
    },
    {
      "fc",
      function()
        vim.g._fff_origin_win = vim.api.nvim_get_current_win()
        require('fff').live_grep({ query = vim.fn.expand("<cword>") })
      end,
      desc = 'Search current word',
    },
  }
}

