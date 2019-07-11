#!/usr/bin/env python
# Sync themes and syntaxes from [bat](https://github.com/sharkdp/bat/tree/master/assets)

import os
from glob import glob
from shutil import copy

if not os.path.exists('../../bat'):
    os.system('git clone https://github.com/sharkdp/bat ../../bat')
else:
    os.system('git -C ../../bat pull')

for syntax_file in glob('../../bat/assets/syntaxes/**/*.sublime-syntax'):
    copy(syntax_file, './syntaxes/')

for theme_file in glob('../../bat/assets/themes/**/*.tmTheme'):
    copy(theme_file, './themes/')

print('Finished.')
