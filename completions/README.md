## Zsh

For zsh completions to work, `zsh/_algo` must be in your `fpath`.

For testing, add this to `~/.zshrc`:

```
export fpath=(/home/anowell/proj/algorithmia-cli/completions/zsh $fpath)
autoload -U compinit
compinit
```

and run `exec zsh` anytime you change the completions file.

## Bash

Source `bash/algo` or
copy it to `/etc/bash_completions.d/` and `exec bash`
(assuming `bash-completion` is installed).

