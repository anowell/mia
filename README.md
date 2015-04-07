Algorithmia CLI Tools
---------------------

Cross-platform CLI for Algorithmia algorithms and data API.

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia-cli.svg)](https://travis-ci.org/algorithmiaio/algorithmia-cli)


## Install

Latest release tarballs are available [here](https://github.com/algorithmiaio/algorithmia-cli/releases) - and simply contain the binary `algo`.

**Debian/Ubuntu**

A `.deb` package is available for installation with `dpkg`

    curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v0.4.0/algorithmia_0.4.0_amd64.deb | sudo dpkg -i

**Arch**

Arch packages available on the AUR as [algorithmia-bin](https://aur.archlinux.org/packages/algorithmia-bin/) and [algorithmia-git](https://aur.archlinux.org/packages/algorithmia-git/), e.g., using `aura` helper.

    # Install from source (requires rust)
    aura -A algorithmia-git

    # Install precompiled binary
    aura -A algorithmia-bin

**OSX**

Simply download and extract the OSX tarball - recommend putting it within your `PATH`

    curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v0.4.0/algo-osx.tar | tar -x

**Windows**

Coming soon - need to setup a Windows build environment...


## Usage

Execute Algorithmia algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ algo -d 19635 kenny/Factor
    {"duration":0.47086329,"result":[3,5,7,11,17]}

Interact with the Algorithmia Data API:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ algo data create anowell/foo create
    CollectionCreated { collection_id: 180, object_id: "01234567-abcd-1234-9876-111111111111", collection_name: "foo", username: "anowell", acl: CollectionAcl { read_w: false, read_g: false, read_u: true, read_a: true } }

    $ algo data upload anowell/foo *.png
    Uploaded /home/anowell/Pictures/collections.png
    Uploaded /home/anowell/Pictures/notif-basic.png
    Uploaded /home/anowell/Pictures/publish_menu.png
    Finished uploading 3 file(s)

    $ algo data show anowell/foo
    CollectionShow { username: "anowell", collection_name: "foo3", files: ["collections.png", "notif-basic.png", "publish_menu.png"] }


Run `algo -h` and `algo data -h` for additional usage help.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

