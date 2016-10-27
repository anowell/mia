## Zsh

For zsh completions to work, `_algo.zsh-completions` must be in your `fpath`.

For testing, add this to `~/.zshrc`:

```
export fpath=(/home/anowell/proj/algorithmia-cli/completions $fpath)
autoload -U compinit
compinit
```

and run `exec zsh` anytime you change the completions file.

## Bash

Coming soon...
