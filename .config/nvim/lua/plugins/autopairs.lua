-- autopairs
-- https://github.com/windwp/nvim-autopairs

return {
  'windwp/nvim-autopairs',
  event = 'InsertEnter',
  -- Optional dependency
  dependencies = { 'hrsh7th/nvim-cmp' },
  config = function()
    local npairs = require 'nvim-autopairs'
    npairs.setup {
      check_ts = true,
      ts_config = {
        go = { 'raw_string_literal' },
      },
    }

    local Rule = require 'nvim-autopairs.rule'
    local ts_conds = require 'nvim-autopairs.ts-conds'

    npairs.add_rules {
      Rule('<', '>', 'go')
        :with_pair(ts_conds.is_ts_node { 'string', 'raw_string_literal_content' })
        :with_move(ts_conds.is_ts_node { 'string', 'raw_string_literal_content' })
        :use_regex(true),
    }

    -- If you want to automatically add `(` after selecting a function or method
    local cmp_autopairs = require 'nvim-autopairs.completion.cmp'
    local cmp = require 'cmp'
    cmp.event:on('confirm_done', cmp_autopairs.on_confirm_done())
  end,
}
