-- Stolen from here:
-- https://github.com/heygarrett/.config/blob/41f747f1dbd29ba8375c051c7b87c4a72806b0a4/nvim/plugin/statusline.lua

-- The statusline is evaluated on every cursor move and redraw, so the branch
-- is cached and refreshed asynchronously instead of spawning git each time.
-- (FocusGained never fires in tmux here: focus-events is off.)
local branch = ''

local function update_branch()
  vim.system({ 'git', 'rev-parse', '--abbrev-ref', 'HEAD' }, { text = true }, function(result)
    local name = result.code == 0 and vim.trim(result.stdout) or ''
    local new = name ~= '' and name .. ' | ' or ''
    if new ~= branch then
      branch = new
      vim.schedule(function()
        vim.cmd.redrawstatus()
      end)
    end
  end)
end

vim.api.nvim_create_autocmd({ 'VimEnter', 'DirChanged', 'FocusGained', 'BufEnter' }, {
  callback = update_branch,
})

local function branch_name()
  return branch
end

local function file_name()
  local root_path = vim.fn.getcwd()
  local root_dir = root_path:match '[^/]+$'
  local home_path = vim.fn.expand '%:~'
  local overlap, _ = home_path:find(root_dir)
  if home_path == '' then
    return root_path:gsub('/Users/ole', '~')
  elseif overlap then
    return home_path:sub(overlap)
  else
    return home_path
  end
end

local function progress()
  if vim.fn.line '.' == 1 then
    return 'top'
  elseif vim.fn.line '.' == vim.fn.line '$' then
    return 'bot'
  else
    local p = vim.fn.line '.' / vim.fn.line '$' * 100
    p = p % 1 >= 0.5 and math.ceil(p) or math.floor(p)
    return string.format('%02d', p) .. '%%'
  end
end

function _G.status_line()
  return ' ' .. '%<' .. branch_name() .. file_name() .. ' ' .. '%h' .. '%m' .. '%=' .. '%y' .. ' ' .. progress() .. ' '
end

vim.opt.statusline = '%{%v:lua.status_line()%}'
