return {
  'rayliwell/tree-sitter-rstml',
  dependencies = { 'nvim-treesitter' },
  build = ':TSUpdate',
  config = function()
    require('tree-sitter-rstml').setup()

    vim.treesitter.query.set("rust_with_rstml", "injections", [[
      (
        (node_attribute
          name: (node_identifier) @_attr_name
          value: (rust_expression (string_literal (string_content) @injection.content))
        )
        (#match? @_attr_name "(^(x-data|x-init|x-for|x-if|x-effect|x-show|x-text)$|(^x-on:|^x-bind:))")
        (#set! injection.include-children)
        (#set! injection.combined)
        (#set! injection.language "javascript")
      )
    ]])

    -- rust analyzers semantic tokens overwrite the injections highlighting. this disables semantic tokens entirely
    vim.api.nvim_create_autocmd('LspAttach', {
      group = vim.api.nvim_create_augroup('UserLspConfig', {}),
      callback = function(ev)
        local client = vim.lsp.get_client_by_id(ev.data.client_id)
        client.server_capabilities.semanticTokensProvider = nil
      end,
    })

    vim.g.vim_treesitter_highlight_timeout = 5000  -- 5 seconds
  end,
}
