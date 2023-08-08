local builtin = require('telescope.builtin')
vim.keymap.set('n', '<leader>ffa', builtin.find_files, {})
vim.keymap.set('n', '<leader>ff', builtin.git_files, {})
vim.keymap.set('n', '<leader>ffg', function()
	builtin.grep_string({ search = vim.fn.input("Grep > ") });
end)

