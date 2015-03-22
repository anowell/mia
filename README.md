Algorithmia CLI Tools
---------------------

Cross-platform CLI for Algorithmia algorithms and data API.

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia_cli.svg)](https://travis-ci.org/algorithmiaio/algorithmia_cli)


## Install

Installation currently just involves dropping 2 binaries (`algo` and `algodata`) anywhere that you can run them (recommend putting them on your `PATH`).

Simply `cd` to the directory where you want them and run:

- Linux:  `curl -L https://github.com/algorithmiaio/algorithmia_cli/releases/download/v0.4.0/algo-linux.tgz | tar -x`
- Mac OS: `curl -L https://github.com/algorithmiaio/algorithmia_cli/releases/download/v0.4.0/algo-osx.tgz | tar -x`
- Windows: ...coming soon - need to setup a Windows build environment...


## Usage

### [algo](src/bin/algo.rs)

A sample CLI tool that uses `exec_raw` to execute algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ algo -d 19635 kenny/Factor
    {"duration":0.47086329,"result":[3,5,7,11,17]}

Run `algo -h` for additional usage help

### [algodata](src/bin/algodata.rs)

A sample CLI tool to interact with the Algorithmia Data API

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ algodata anowell/foo create
    CollectionCreated { collection_id: 180, object_id: "01234567-abcd-1234-9876-111111111111", collection_name: "foo", username: "anowell", acl: CollectionAcl { read_w: false, read_g: false, read_u: true, read_a: true } }

    $ algodata anowell/foo upload *.png
    Uploaded /home/anowell/Pictures/collections.png
    Uploaded /home/anowell/Pictures/notif-basic.png
    Uploaded /home/anowell/Pictures/publish_menu.png
    Finished uploading 3 file(s)

    $ algodata anowell/foo
    CollectionShow { username: "anowell", collection_name: "foo3", files: ["collections.png", "notif-basic.png", "publish_menu.png"] }


Run `algodata -h` for additional usage help.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

