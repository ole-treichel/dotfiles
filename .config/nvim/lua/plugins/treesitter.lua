return {
  'nvim-treesitter/nvim-treesitter',
  build = ':TSUpdate',

  config = function()

    local parser_config = require('nvim-treesitter.parsers').get_parser_configs()
    parser_config.datastar = {
      install_info = {
        url = "/home/ole/.local/share/tree-sitter-datastar",
        files = {"src/parser.c", "src/scanner.c"},
        branch = "main",
        generate_requires_npm = false,
        requires_generate_from_grammar = false,
      },
    }

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
  end,
}
