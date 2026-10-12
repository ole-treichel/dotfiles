-- Pick up external edits (git, claude, formatters) without prompts.
-- 1. A directory watcher runs :checktime as soon as a loaded file changes on
--    disk, so unmodified buffers reload before you start typing in them.
--    Directories, not files: atomic writes (tmp + rename) would kill a file watch.
-- 2. If the buffer has unsaved changes, nvim reloads it anyway and the unsaved
--    changes are 3-way merged back on top (base = content at last read/write).
--    Conflicts become git-style markers. `u` restores the pre-merge buffer.
-- See docs/nvim-autoread.md.
vim.opt.autoread = true

local base = {} -- bufnr -> lines as last read from / written to disk
local pending = {} -- bufnr -> state captured before a conflict reload
local watched = {} -- realpath -> { [bufnr] = true }
local watchers = {} -- dir -> uv fs_event handle
local timer = vim.uv.new_timer()

local function checktime()
  -- :checktime fails in the cmdline window; retry once the user is out.
  if vim.fn.getcmdwintype() ~= '' or vim.fn.mode():sub(1, 1) == 'c' then
    timer:start(500, 0, vim.schedule_wrap(checktime))
    return
  end
  vim.cmd.checktime()
end

local function on_change(dir, filename)
  if filename and not watched[dir .. '/' .. filename] then return end
  -- debounce bursts (git checkout, rebase) into one check
  timer:start(100, 0, vim.schedule_wrap(checktime))
end

local function unwatch(buf)
  for path, bufs in pairs(watched) do
    bufs[buf] = nil
    if next(bufs) == nil then watched[path] = nil end
  end
  for dir, handle in pairs(watchers) do
    local used = false
    for path in pairs(watched) do
      if vim.fs.dirname(path) == dir then used = true break end
    end
    if not used then
      handle:stop()
      handle:close()
      watchers[dir] = nil
    end
  end
end

local function watch(buf)
  unwatch(buf)
  if vim.bo[buf].buftype ~= '' then return end
  -- resolve symlinks (~/.bashrc -> repo): events fire in the target's dir
  local path = vim.uv.fs_realpath(vim.api.nvim_buf_get_name(buf))
  if not path then return end
  local dir = vim.fs.dirname(path)
  watched[path] = watched[path] or {}
  watched[path][buf] = true
  if watchers[dir] then return end
  local handle = vim.uv.new_fs_event()
  if not handle then return end
  handle:start(dir, {}, function(err, filename)
    if not err then on_change(dir, filename) end
  end)
  watchers[dir] = handle
end

local function hunks(a, b)
  return vim.diff(table.concat(a, '\n') .. '\n', table.concat(b, '\n') .. '\n', { result_type = 'indices' }) --[[@as integer[][] ]]
end

-- Buffer was just reloaded from disk; put the unsaved changes back on top.
local function merge(buf, p)
  local theirs = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  local files = { vim.fn.tempname(), vim.fn.tempname(), vim.fn.tempname() }
  vim.fn.writefile(p.ours, files[1])
  vim.fn.writefile(p.base, files[2])
  vim.fn.writefile(theirs, files[3])
  local res = vim.system({ 'git', 'merge-file', '-p', '-L', 'buffer', '-L', 'base', '-L', 'disk',
    files[1], files[2], files[3] }, { text = true }):wait()
  for _, f in ipairs(files) do os.remove(f) end

  local short = vim.fn.fnamemodify(vim.api.nvim_buf_get_name(buf), ':~:.')
  if res.code < 0 or res.code > 127 then
    -- reload is in the undo history: `u` brings the unsaved changes back
    vim.notify('autoread: merge failed for ' .. short .. ', reloaded from disk (u to undo)', vim.log.levels.ERROR)
    return
  end
  local merged = vim.split(res.stdout, '\n', { plain = true })
  if merged[#merged] == '' then table.remove(merged) end

  -- apply as minimal hunks, bottom-up, so extmarks stay on their text;
  -- undojoin folds them into the reload's undo step
  local h = hunks(theirs, merged)
  vim.api.nvim_buf_call(buf, function()
    for i = #h, 1, -1 do
      local sa, ca, sb, cb = unpack(h[i])
      local start = ca == 0 and sa or sa - 1
      pcall(vim.cmd.undojoin)
      vim.api.nvim_buf_set_lines(buf, start, start + ca, false, vim.list_slice(merged, sb, sb + cb - 1))
    end
  end)

  -- reload keeps cursor line numbers; shift them by what changed above
  local shifts = hunks(p.ours, merged)
  for win, pos in pairs(p.cursors) do
    if vim.api.nvim_win_is_valid(win) and vim.api.nvim_win_get_buf(win) == buf then
      local line = pos[1]
      for _, s in ipairs(shifts) do
        -- hunk ends above the cursor (pure insertions: inserted after line s[1])
        if s[1] + math.max(s[2], 1) <= pos[1] then
          line = line + s[4] - s[2]
        end
      end
      pcall(vim.api.nvim_win_set_cursor, win, { math.max(1, line), pos[2] })
    end
  end

  if res.code > 0 then
    vim.notify(('autoread: %d conflict(s) in %s, markers inserted'):format(res.code, short), vim.log.levels.WARN)
  elseif #h > 0 then
    vim.notify('autoread: merged disk changes into ' .. short)
  end
end

local group = vim.api.nvim_create_augroup('autoread', {})

vim.api.nvim_create_autocmd({ 'BufReadPost', 'BufWritePost' }, {
  group = group,
  callback = function(ev)
    base[ev.buf] = vim.api.nvim_buf_get_lines(ev.buf, 0, -1, false)
    watch(ev.buf)
  end,
})

vim.api.nvim_create_autocmd('BufUnload', {
  group = group,
  callback = function(ev)
    base[ev.buf] = nil
    unwatch(ev.buf)
  end,
})

-- fallback for files the watcher can't see (e.g. dir created after opening).
-- FocusGained already runs :checktime natively.
vim.api.nvim_create_autocmd('BufEnter', {
  group = group,
  callback = function()
    if vim.fn.getcmdwintype() == '' then vim.cmd.checktime() end
  end,
})

-- Only fires when nvim can't reload silently, i.e. the buffer has unsaved changes.
-- Reloading (instead of patching the buffer in place) matters: it resets the
-- mtime nvim compares on :w, so saving afterwards doesn't warn either.
vim.api.nvim_create_autocmd('FileChangedShell', {
  group = group,
  callback = function(ev)
    if vim.v.fcs_reason ~= 'conflict' or not base[ev.buf] then
      vim.v.fcs_choice = 'ask' -- default behaviour (deleted, mode change, ...)
      return
    end
    local cursors = {}
    for _, win in ipairs(vim.fn.win_findbuf(ev.buf)) do
      cursors[win] = vim.api.nvim_win_get_cursor(win)
    end
    pending[ev.buf] = { ours = vim.api.nvim_buf_get_lines(ev.buf, 0, -1, false), base = base[ev.buf], cursors = cursors }
    vim.v.fcs_choice = 'reload'
  end,
})

vim.api.nvim_create_autocmd('FileChangedShellPost', {
  group = group,
  callback = function(ev)
    local p = pending[ev.buf]
    pending[ev.buf] = nil
    if p then merge(ev.buf, p) end
  end,
})
