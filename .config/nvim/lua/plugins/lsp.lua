return {
  'neovim/nvim-lspconfig',

  config = function()
    -- Setup language servers.
    local lspconfig = require 'lspconfig'

    lspconfig.tsserver.setup {}
    lspconfig.rust_analyzer.setup {}
    lspconfig.templ.setup {}
    lspconfig.html.setup {
      filetypes = { 'html', 'templ' },
    }
    lspconfig.gopls.setup {}

    -- Use LspAttach autocommand to only map the following keys
    -- after the language server attaches to the current buffer
    vim.api.nvim_create_autocmd('LspAttach', {
      group = vim.api.nvim_create_augroup('UserLspConfig', {}),
      callback = function(ev)
        -- Enable completion triggered by <c-x><c-o>
        vim.bo[ev.buf].omnifunc = 'v:lua.vim.lsp.omnifunc'

        -- Buffer local mappings.
        -- See `:help vim.lsp.*` for documentation on any of the below functions
        local opts = { buffer = ev.buf }
        vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
        vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
        vim.keymap.set('n', '<space>rn', vim.lsp.buf.rename, opts)
      end,
    })
  end,
}
