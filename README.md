Algorithmia CLI Tools
---------------------

Cross-platform CLI for Algorithmia algorithms and data API.

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia-cli.svg)](https://travis-ci.org/algorithmiaio/algorithmia-cli)


## Install

Latest releases can be found [here](https://github.com/algorithmiaio/algorithmia-cli/releases/latest).

**Debian/Ubuntu**

A `.deb` package is available for installation with `dpkg`

```bash
curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v1.0.0-beta.2/algorithmia_1.0.0-beta.2_amd64.deb | sudo dpkg -i
```

**Arch**

Arch packages available on the AUR as [algorithmia-bin](https://aur4.archlinux.org/packages/algorithmia-bin/) and [algorithmia-git](https://aur4.archlinux.org/packages/algorithmia-git/), e.g., using `aura` helper:

```bash
aura -A algorithmia-bin
```

**OSX**

Simply download and extract the OSX tarball - recommend putting it within your `PATH`

```bash
curl -L https://github.com/algorithmiaio/algorithmia-cli/releases/download/v1.0.0-beta.2/algorithmia_osx.tar.gz | tar -xz
sudo mv algo /usr/local/bin/
```

**Windows (64-bit)**

Install [OpenSSL for Windows](https://slproweb.com/products/Win32OpenSSL.html)

Download and extract the [latest Windows zip file](https://github.com/algorithmiaio/algorithmia-cli/releases/download/v1.0.0-beta.2/algorithmia_win64.zip) - recommend putting it within your `PATH`


## Usage

Configure auth:

    $ algo auth
    Configuring authentication for 'default' profile
    Enter API Key (prefixed with 'sim'):
    Profile is ready to use. Test with 'algo ls'

Execute Algorithmia algorithms:

    $ algo run kenny/factor -d 19635
    [3,5,7,11,17]
    $ algo run kenny/factor -d 19635 --response-body
    {"result":[3,5,7,11,17],"metadata":{"content_type":"json","duration":0.001427314}}

Interact with the Algorithmia Data API:

    $ algo mkdir .my/foo
    Created directory data://.my/foo

    $ algo cp *.png data://.my/foo
    Uploaded data://.my/foo/collections.png
    Uploaded data://.my/foo/notif-basic.png
    Uploaded data://.my/foo/publish_menu.png
    Finished uploading 3 file(s)

    $ algo ls .my/foo
    collections.png  notif-basic.png  publish_menu.png

Run `algo --help` for additional usage help.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test
