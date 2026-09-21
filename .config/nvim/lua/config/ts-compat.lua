-- nvim-treesitter master (archived, does not support Neovim 0.12) registers its
-- query predicates/directives with `all = false`, i.e. expecting `match[id]` to be
-- a single TSNode. Neovim 0.12 removed that compat wrapper, so the handlers now get
-- a TSNode[] and die on `node:range()` -- e.g. on every markdown code fence, which
-- breaks LSP hover.
--
-- Re-wrap only those handlers, by name, until the config moves to the main branch.
-- See docs/nvim-treesitter-0-12.md

if vim.fn.has 'nvim-0.12' == 0 then
  return
end

local legacy = {
  ['nth?'] = true,
  ['is?'] = true,
  ['kind-eq?'] = true,
  ['set-lang-from-mimetype!'] = true,
  ['set-lang-from-info-string!'] = true,
  ['downcase!'] = true,
}

local query = vim.treesitter.query

for _, register in ipairs { 'add_predicate', 'add_directive' } do
  local orig = query[register]

  query[register] = function(name, handler, opts)
    if not legacy[name] then
      return orig(name, handler, opts)
    end

    return orig(name, function(match, ...)
      local last = {}
      for id, nodes in pairs(match) do
        last[id] = nodes[#nodes]
      end
      return handler(last, ...)
    end, opts)
  end
end
