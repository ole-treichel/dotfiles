return {
  'neovim/nvim-lspconfig',

  config = function()
    -- Setup language servers.
    local lspconfig = require 'lspconfig'

    lspconfig.tsserver.setup {}
    lspconfig.rust_analyzer.setup {}
    lspconfig.templ.setup {}
    lspconfig.tailwindcss.setup({
      filetypes = { "templ", "javascript", "react" },
      init_options = { userLanguages = { templ = "html" } },
    })
    lspconfig.html.setup {
      filetypes = { 'html', 'templ' },
    }
    lspconfig.htmx.setup({
      filetypes = { "javascript", "react", "html", "templ" },
    })
    lspconfig.gopls.setup {}

    vim.api.nvim_create_autocmd("BufWritePre", {
      pattern = "*.go",
      callback = function()
        local params = vim.lsp.util.make_range_params()
        params.context = {only = {"source.organizeImports"}}
        -- buf_request_sync defaults to a 1000ms timeout. Depending on your
        -- machine and codebase, you may want longer. Add an additional
        -- argument after params if you find that you have to write the file
        -- twice for changes to be saved.
        -- E.g., vim.lsp.buf_request_sync(0, "textDocument/codeAction", params, 3000)
        local result = vim.lsp.buf_request_sync(0, "textDocument/codeAction", params)
        for cid, res in pairs(result or {}) do
          for _, r in pairs(res.result or {}) do
            if r.edit then
              local enc = (vim.lsp.get_client_by_id(cid) or {}).offset_encoding or "utf-16"
              vim.lsp.util.apply_workspace_edit(r.edit, enc)
            end
          end
        end
        vim.lsp.buf.format({async = false})
      end
    })

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
