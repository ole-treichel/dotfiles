return {
  'neovim/nvim-lspconfig',

  config = function()
    local capabilities = require('cmp_nvim_lsp').default_capabilities()
    capabilities.textDocument.completion.completionItem.snippetSupport = true

    -- Set default capabilities for all servers
    vim.lsp.config('*', {
      capabilities = capabilities,
    })

    -- TypeScript (only for Node.js projects, not Deno)
    vim.lsp.config('ts_ls', {
      root_dir = function(bufnr, on_dir)
        if vim.fs.root(bufnr, { 'deno.json', 'deno.jsonc' }) then
          return
        end
        on_dir(vim.fs.root(bufnr, { 'package.json' }))
      end,
    })

    -- Deno
    vim.lsp.config('denols', {
      root_markers = { 'deno.json', 'deno.jsonc' },
      init_options = {
        lint = true,
        unstable = true,
      },
    })

    -- HTML (with extended filetypes)
    vim.lsp.config('html', {
      filetypes = {
        "javascript",
        "javascriptreact",
        "javascript.jsx",
        "typescript",
        "typescriptreact",
        "typescript.tsx",
        "rust",
      },
    })

    -- Enable all servers
    vim.lsp.enable({
      'ts_ls',
      'denols',
      'cssls',
      'rust_analyzer',
      'templ',
      'html',
      'gopls',
      'jsonls',
    })

    vim.api.nvim_create_autocmd('BufWritePre', {
      pattern = '*.go',
      callback = function()
        local params = vim.lsp.util.make_range_params()
        params.context = { only = { 'source.organizeImports' } }
        -- buf_request_sync defaults to a 1000ms timeout. Depending on your
        -- machine and codebase, you may want longer. Add an additional
        -- argument after params if you find that you have to write the file
        -- twice for changes to be saved.
        -- E.g., vim.lsp.buf_request_sync(0, "textDocument/codeAction", params, 3000)
        local result = vim.lsp.buf_request_sync(0, 'textDocument/codeAction', params)
        for cid, res in pairs(result or {}) do
          for _, r in pairs(res.result or {}) do
            if r.edit then
              local enc = (vim.lsp.get_client_by_id(cid) or {}).offset_encoding or 'utf-16'
              vim.lsp.util.apply_workspace_edit(r.edit, enc)
            end
          end
        end
        vim.lsp.buf.format { async = false }
      end,
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
        vim.keymap.set('n', 'grr', vim.lsp.buf.references)
        vim.keymap.set('n', 'grn', vim.lsp.buf.rename)
        vim.keymap.set({ 'i', 'n' }, '<C-k>', function() require('cmp').complete() end, opts)
      end,
    })

    -- https://github.com/neovim/neovim/issues/30985#issuecomment-2447329525
    for _, method in ipairs({ 'textDocument/diagnostic', 'workspace/diagnostic' }) do
        local default_diagnostic_handler = vim.lsp.handlers[method]
        vim.lsp.handlers[method] = function(err, result, context, config)
            if err ~= nil and err.code == -32802 then
                return
            end
            return default_diagnostic_handler(err, result, context, config)
        end
    end

    vim.api.nvim_create_autocmd('BufWritePre', {
      pattern = { '*.ts', '*.tsx', '*.js', '*.jsx', '*.json' },
      callback = function(args)
        -- Check if we're in a Deno project (has deno.json or deno.jsonc)
        local file_dir = vim.fn.expand('%:p:h')
        local is_deno_project = false
        local check_dir = file_dir
        while check_dir ~= '/' do
          if vim.fn.filereadable(check_dir .. '/deno.json') == 1 or
             vim.fn.filereadable(check_dir .. '/deno.jsonc') == 1 then
            is_deno_project = true
            break
          end
          check_dir = vim.fn.fnamemodify(check_dir, ':h')
        end
        if is_deno_project then
          -- Use LSP formatting for Deno files
          vim.lsp.buf.format({
            async = false,
            filter = function(client)
              return client.name == "denols"
            end
          })
        end
      end,
    })
  end,
}
