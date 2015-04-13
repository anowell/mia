Algorithmia CLI Tools
---------------------

Cross-platform CLI for Algorithmia algorithms and data API.

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia-cli.svg)](https://travis-ci.org/algorithmiaio/algorithmia-cli)


## Install

Latest release tarballs are available [here](https://github.com/algorithmiaio/algorithmia-cli/releases) - and simply contain the binary `algo`.

**Debian/Ubuntu**

A `.deb` package is available for installation with `dpkg`

    curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v0.6.0/algorithmia_0.6.0_amd64.deb | sudo dpkg -i

**Arch**

Arch packages available on the AUR as [algorithmia-bin](https://aur.archlinux.org/packages/algorithmia-bin/) and [algorithmia-git](https://aur.archlinux.org/packages/algorithmia-git/), e.g., using `aura` helper.

    # Install from source (requires rust)
    aura -A algorithmia-git

    # Install precompiled binary
    aura -A algorithmia-bin

**OSX**

Simply download and extract the OSX tarball - recommend putting it within your `PATH`

    curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v0.6.0/algo-osx.tar | tar -x

**Windows**

Coming soon - need to setup a Windows build environment...


## Usage

Execute Algorithmia algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ algo run kenny/Factor -d 19635
    {"duration":0.47086329,"result":[3,5,7,11,17]}

Interact with the Algorithmia Data API:

    $ algo create anowell/foo create
    Created collection anowell/foo

    $ algo upload anowell/foo *.png
    Uploaded /home/anowell/Pictures/collections.png
    Uploaded /home/anowell/Pictures/notif-basic.png
    Uploaded /home/anowell/Pictures/publish_menu.png
    Finished uploading 3 file(s)

    $ algo ls anowell/foo
    anowell/foo - 3 files
    collections.png
    notif-basic.png
    publish_menu.png

Run `algo --help` for additional usage help.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

