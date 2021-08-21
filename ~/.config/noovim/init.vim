if &compatible
    set nocompatible               " Be iMproved
endif

" Disable HTML polyglot due to dodgy inline-js indents
let g:polyglot_disabled = ['html'] " breaks java

call plug#begin('~/.vim/plugged')

set updatetime=300

" Autocompletion - CoC
Plug 'neoclide/coc.nvim', {'branch': 'release'}

" Statusbar
Plug 'hoob3rt/lualine.nvim'

" Colourscheme
Plug 'folke/tokyonight.nvim'

" Plug 'antoinemadec/openrgb.nvim', {'do': 'UpdateRemotePlugins'}
Plug 'folke/trouble.nvim'

" Git
Plug 'tpope/vim-fugitive' " Wrapper
Plug 'mhinz/vim-signify'
Plug 'tveskag/nvim-blame-line' " Git blame

" Improved syntax
Plug 'sheerun/vim-polyglot'

" Ranger
Plug 'rbgrouleff/bclose.vim'
Plug 'francoiscabrol/ranger.vim'

" Start page
Plug 'mhinz/vim-startify'

" Icons
Plug 'ryanoasis/vim-devicons'
Plug 'kyazdani42/nvim-web-devicons'

" Better top bar
" Plug 'romgrk/barbar.nvim'

" Colour code highlighting
Plug 'ap/vim-css-color'

" Zen mode
Plug 'junegunn/goyo.vim'

Plug 'honza/vim-snippets'
Plug 'SirVer/ultisnips'

" Todo highlighting
Plug 'nvim-lua/plenary.nvim'
Plug 'folke/todo-comments.nvim'
" Pairs-closing
" Plug 'jiangmiao/auto-pairs'

" Fuzzy finder
Plug 'fannheyward/telescope-coc.nvim'
Plug 'nvim-telescope/telescope.nvim'
" Plug 'ctrlpvim/ctrlp.vim'
" Plug 'junegunn/fzf', { 'do': { -> fzf#install() } }

" Ron highlighting
Plug 'ron-rs/ron.vim'

" Indent guides
Plug 'yggdroot/indentline'

" UNIX Helper
Plug 'tpope/vim-eunuch'

" Tagbar
Plug 'preservim/tagbar'

" GraphViz
Plug 'wannesm/wmgraphviz.vim'

" Ale
Plug 'dense-analysis/ale'

" Marks in gutter
Plug 'kshenoy/vim-signature'

" Haskell
Plug 'neovimhaskell/haskell-vim'

call plug#end()

" Required:
filetype plugin indent on
syntax enable

let g:python3_host_prog = '/usr/bin/python'

" Plugin Config -----------

" Snippets
let g:UltiSnipsExpandTrigger="<nop>"

" --------------------   Coc    --------------------
inoremap <silent><expr> <c-space> coc#refresh()
" Tab stuff
inoremap <silent><expr> <TAB>
      \ pumvisible() ? "\<C-n>" :
      \ <SID>check_back_space() ? "\<TAB>" :
      \ coc#refresh()
inoremap <expr><S-TAB> pumvisible() ? "\<C-p>" : "\<C-h>"

let g:coc_snippet_next = '<tab>'

function! s:check_back_space() abort
  let col = col('.') - 1
  return !col || getline('.')[col - 1]  =~# '\s'
endfunction

inoremap <expr> <cr> pumvisible() ? "\<C-y>" : "\<C-g>u\<CR>"
inoremap <silent><expr> <cr> pumvisible() ? coc#_select_confirm() : "\<C-g>u\<CR>"
nnoremap <Leader>p :CocCommand<CR>
nnoremap <Leader>t :Telescope<CR>

nnoremap <Leader><Leader> :CocAction<CR>

inoremap <silent><expr> <cr> pumvisible() ? coc#_select_confirm()
            \: "\<C-g>u\<CR>\<c-r>=coc#on_enter()\<CR>"

" ---
nmap <Leader>rn <Plug>(coc-rename)
nmap <silent>gd <Plug>(coc-definition)
nmap <silent>gr <Plug>(coc-references)

" --------------------    Git   --------------------
nnoremap ga :Git add %<CR>
nnoremap gc :Git commit<CR>
nnoremap gs :Git status<CR>

" -------------------- Startify --------------------
let g:startify_custom_header = [
            \ '     █████████ ██████████ ██████████ ███    ███ ███ ███████████████',
            \ '     ███▀▀▀███ ███    ███ ███    ███ ███    ███ ███ ███▀▀▀███▀▀▀███',
            \ '     ███   ███ ███    ███ ███    ███ ███    ███ ███ ███   ███   ███',
            \ '     ███   ███ ███▄▄▄     ███    ███ ███    ███ ███ ███   ███   ███',
            \ '     ███   ███ ███▀▀▀     ███    ███ ███    ███ ███ ███   ███   ███',
            \ '     ███   ███ ███    ███ ███    ███ ███    ███ ███ ███   ███   ███',
            \ '     ███   ███ ███    ███ ███    ███ ███    ███ ███ ███   ███   ███',
            \ '     ███   ███ ██████████ ██████████ ██████████ ███ ███   ███   ███',
            \ ]

let g:startify_bookmarks = ['~/.config/nvim/init.vim']

" returns all modified files of the current git repo
" `2>/dev/null` makes the command fail quietly, so that when we are not
" in a git repo, the list will be empty
function! s:gitModified()
    let files = systemlist('git ls-files -m 2>/dev/null')
    return map(files, "{'line': v:val, 'path': v:val}")
endfunction

" same as above, but show untracked files, honouring .gitignore
function! s:gitUntracked()
    let files = systemlist('git ls-files -o --exclude-standard 2>/dev/null')
    return map(files, "{'line': v:val, 'path': v:val}")
endfunction

let g:startify_lists = [
        \ { 'type': 'files',     'header': ['   MRU']            },
        \ { 'type': 'dir',       'header': ['   MRU '. getcwd()] },
        \ { 'type': 'sessions',  'header': ['   Sessions']       },
        \ { 'type': 'bookmarks', 'header': ['   Bookmarks']      },
        \ { 'type': function('s:gitModified'),  'header': ['   Git Modified']},
        \ { 'type': function('s:gitUntracked'), 'header': ['   Git Untracked']},
        \ { 'type': 'commands',  'header': ['   Commands']       },
        \ ]

"--------------------   Ale  -------------------- 
" let g:ale_disable_lsp = 1
let g:ale_echo_cursor= 0
let g:ale_cursor_detail=1
let g:ale_floating_preview = 1
let g:ale_floating_window_border = ['║', '═', '╔', '╗', '╝', '╚']
let g:ale_linters = {
    \ 'python': [],
    \ 'haskell': [],
    \ 'rust': [],
    \ 'typescript': []
    \}
let g:ale_rust_rls_toolchain = 'stable'

" Django
set wildignore +=*/staticfiles/*,*/node_modules/*,*/env/*

" Colour Scheme
set termguicolors
let g:tokyonight_style = 'storm'
colorscheme tokyonight

" Gutter marks
highlight SignatureMarkText guifg=#5CCFE6

" Move cursor to new split
set splitright
set splitbelow

" --------------

set nowrap " Disable text-wrap
set number " Show line numbers

autocmd BufRead,BufNewFile * setlocal signcolumn=number " Enable signcolumn by default
highlight clear SignColumn " Clear highlight from sign column

runtime mswin.vim " Enable mswin style bindings - until I break the habit of <C-s>

tnoremap <C-[> <C-\><C-n> " Change mapping to make terminal easier to exit

set tabstop=4 shiftwidth=4 expandtab

set mouse=a " Enable mouse support

set clipboard+=unnamedplus " Always use system clipboard

set cursorline " Make cursor easier to find

nnoremap <del> "_x " Make the delete button not yank

" set noequalalways " Disable equalalways so splits aren't resized to be equal when another split is made

" Custom filetype stuff
autocmd BufNewFile,BufRead *.pw set filetype=pw

set scrolloff=10

" Function to delete all hidden buffers
function DeleteHiddenBuffers()
    let tpbl=[]
    call map(range(1, tabpagenr('$')), 'extend(tpbl, tabpagebuflist(v:val))')
    for buf in filter(range(1, bufnr('$')), 'bufexists(v:val) && index(tpbl, v:val)==-1')
        silent execute 'bwipeout' buf
    endfor
endfunction

nnoremap <silent> <c-K> :call <SID>show_documentation()<CR>

function! s:show_documentation()
  if (index(['vim','help'], &filetype) >= 0)
    execute 'h '.expand('<cword>')
  elseif (coc#rpc#ready())
    call CocActionAsync('doHover')
  else
    execute '!' . &keywordprg . " " . expand('<cword>')
  endif
endfunction

" Fuzzy finder
if executable('fish')
    " use fish for embedded terminals
    set shell=fish
    " use bash for else
    let $SHELL = 'bash'
endif

nnoremap <silent> <C-p> :<C-u>Telescope find_files<CR>
" let g:fzf_preview_default_fzf_options = { '--reverse': v:true, '--preview-window': 'nowrap' }
" let g:fzf_preview_use_dev_icons = 1

lua << EOF
require('lualine').setup({
    options = {
        theme = 'tokyonight'
    },
    tabline = {
        lualine_a = {'hostname'},
        lualine_b = {'branch'},
        lualine_c = {'filename'},
        lualine_x = {'diff'},
        lualine_y = {},
        lualine_z = {}
    },

    sections = {
        lualine_a = {'mode'},
        lualine_b = {'branch',
                        { 'diagnostics', sources = {'coc', 'ale'} }
                    },
        lualine_c = {'g:coc_status', 'filename'},
        lualine_x = {'encoding', 'fileformat', 'filetype'},
        lualine_y = {'progress'},
        lualine_z = {'location'}
    },

    extensions = {'fugitive'}
})

require("trouble").setup {
    height = 10, -- height of the trouble list
    icons = true, -- use devicons for filenames
    mode = "loclist", -- "lsp_workspace_diagnostics", "lsp_document_diagnostics", "quickfix", "lsp_references", "loclist"
    fold_open = "", -- icon used for open folds
    fold_closed = "", -- icon used for closed folds
    action_keys = { -- key mappings for actions in the trouble list
        close = "q", -- close the list
        cancel = "<esc>", -- cancel the preview and get back to your last window / buffer / cursor
        refresh = "r", -- manually refresh
        jump = {"<cr>", "<tab>"}, -- jump to the diagnostic or open / close folds
        jump_close = {"o"}, -- jump to the diagnostic and close the list
        toggle_mode = "m", -- toggle between "workspace" and "document" diagnostics mode
        toggle_preview = "P", -- toggle auto_preview
        hover = "K", -- opens a small poup with the full multiline message
        preview = "p", -- preview the diagnostic location
        close_folds = {"zM", "zm"}, -- close all folds
        open_folds = {"zR", "zr"}, -- open all folds
        toggle_fold = {"zA", "za"}, -- toggle fold of current file
        previous = "k", -- preview item
        next = "j" -- next item
    },
    indent_lines = false, -- add an indent guide below the fold icons
    auto_open = false, -- automatically open the list when you have diagnostics
    auto_close = false, -- automatically close the list when you have no diagnostics
    auto_preview = true, -- automatyically preview the location of the diagnostic. <esc> to close preview and go back to last window
    auto_fold = false, -- automatically fold a file trouble list at creation
    signs = {
        -- icons / text used for a diagnostic
        error = "",
        warning = "",
        hint = "",
        information = "",
        other = "﫠"
    },
    use_lsp_diagnostic_signs = false -- enabling this will use the signs defined in your lsp client
}

require('todo-comments').setup {
}

require('telescope').setup {
    defaults = {
        file_ignore_patterns = { 'node_modules', '__pycache__', '**/migrations', 'staticfiles', 'env' }
    }
}
require('telescope').load_extension('coc')
EOF
