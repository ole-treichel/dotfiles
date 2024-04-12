local lsp = require("lsp-zero")

lsp.preset("recommended")

lsp.ensure_installed({
  'gopls',
  'html',
  'templ',
  'tsserver',
  'rust_analyzer',
})

-- Fix Undefined global 'vim'
lsp.nvim_workspace()

local cmp = require('cmp')
local cmp_select = {behavior = cmp.SelectBehavior.Select}
local lspconfig = require('lspconfig')

local has_words_before = function()
  unpack = unpack or table.unpack
  local line, col = unpack(vim.api.nvim_win_get_cursor(0))
  return col ~= 0 and vim.api.nvim_buf_get_lines(0, line - 1, line, true)[1]:sub(col, col):match("%s") == nil
end

local feedkey = function(key, mode)
  vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes(key, true, true, true), mode, true)
end

local cmp_mappings = lsp.defaults.cmp_mappings({
	['<C-b>'] = cmp.mapping.scroll_docs(-4),
	['<C-f>'] = cmp.mapping.scroll_docs(4),
	["<C-Space>"] = cmp.mapping.complete(),
	["<C-e>"] = cmp.mapping.close(),
	["<CR>"] = cmp.mapping.confirm({
		behavior = cmp.ConfirmBehavior.Replace,
		select = false,
	}),

  ["<Tab>"] = cmp.mapping(function(fallback)
    if cmp.visible() then
      cmp.select_next_item()
    elseif vim.fn["vsnip#available"](1) == 1 then
      feedkey("<Plug>(vsnip-expand-or-jump)", "")
    elseif has_words_before() then
      cmp.complete()
    else
      fallback() -- The fallback function sends a already mapped key. In this case, it's probably `<Tab>`.
    end
  end, { "i", "s" }),

  ["<S-Tab>"] = cmp.mapping(function()
    if cmp.visible() then
      cmp.select_prev_item()
    elseif vim.fn["vsnip#jumpable"](-1) == 1 then
      feedkey("<Plug>(vsnip-jump-prev)", "")
    end
  end, { "i", "s" }),
});

lsp.setup_nvim_cmp({
  -- See issues here: https://www.reddit.com/r/neovim/comments/1139ngo/how_to_make_nvimcmp_menu_selection_start_at_the/
  completion = { completeopt = "noselect" },
  preselect = cmp.PreselectMode.None,
  mapping = cmp_mappings
})

lsp.set_preferences({
    suggest_lsp_servers = false,
    sign_icons = {
        error = 'E',
        warn = 'W',
        hint = 'H',
        info = 'I'
    }
})

lsp.on_attach(function(client, bufnr)
  local opts = {buffer = bufnr, remap = false}

  vim.keymap.set("n", "gd", function() vim.lsp.buf.definition() end, opts)
  vim.keymap.set("n", "<leader>ca", function() vim.lsp.buf.code_action() end, opts)
end)

lspconfig.html.setup({
    filetypes = { "html", "templ" },
})

vim.filetype.add({ extension = { templ = "templ" } })

lsp.setup()

vim.diagnostic.config({
    virtual_text = true
})

