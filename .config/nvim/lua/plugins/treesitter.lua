return {
  'nvim-treesitter/nvim-treesitter',
  build = ':TSUpdate',

  config = function()
    require('nvim-treesitter.configs').setup {
      -- A list of parser names, or "all" (the five listed parsers should always be installed)
      ensure_installed = { 'lua', 'vimdoc', 'rust', 'typescript', 'html', 'css', 'javascript', 'templ' },

      -- Install parsers synchronously (only applied to `ensure_installed`)
      sync_install = false,

      -- Automatically install missing parsers when entering buffer
      -- Recommendation: set to false if you don't have `tree-sitter` CLI installed locally
      auto_install = true,

      indent = {
        enable = true,

        additional_vim_regex_highlighting = { 'tsx' },
      },

      highlight = {
        enable = true,

        -- Setting this to true will run `:h syntax` and tree-sitter at the same time.
        -- Set this to `true` if you depend on 'syntax' being enabled (like for indentation).
        -- Using this option may slow down your editor, and you may see some duplicate highlights.
        -- Instead of true it can also be a list of languages
        additional_vim_regex_highlighting = false,
      },
    }

    vim.treesitter.query.set(
      'go',
      'injections',
      [[
      ((const_spec
        (comment) @_html_comment
        value: (expression_list
          (raw_string_literal
            (raw_string_literal_content) @injection.content)))
        (#eq? @_html_comment "/* html */")
        (#set! injection.combined true)
        (#set! injection.language "html"))

      ]]
    )
  end,
}
