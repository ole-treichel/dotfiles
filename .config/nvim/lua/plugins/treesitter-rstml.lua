return {
  'rayliwell/tree-sitter-rstml',
  dependencies = { 'nvim-treesitter' },
  build = ':TSUpdate',
  config = function()
    require('tree-sitter-rstml').setup()

    vim.treesitter.query.set(
      'rust_with_rstml',
      'injections',
      [[
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
    ]]
    )

    -- semantic tokens seem to override tree sitter in general. by setting to lower than 100, tree sitter is not overriden by the lsp
    -- https://github.com/NvChad/NvChad/issues/1907#issuecomment-1501269595
    vim.highlight.priorities.semantic_tokens = 95

    vim.g.vim_treesitter_highlight_timeout = 5000 -- 5 seconds
  end,
}
