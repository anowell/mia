## 2.0.0-alpha.1
- Rename to mia (no longer official client)

## 1.0.1 (Mar 31, 2017)
- Fix duplicate printing of stacktraces and some error causes
- Add color for many errors
- [serve] Fix error parsing
- [runlocal] Don't hang if build fails
- [runlocal] Allow skipping build

## 1.0.0 (Mar 30, 2017)
- Fix I/O hanging on OSX
- Revamp error messages
- Migrate config to ~/.algorithmia/config
- [auth] Prompts for API endpoint (enterprise)
- [runlocal] Real-time algorithm stdout/stderr
- [serve] Added --no-build arg and better logging

## 1.0.0-beta.4 (Nov 21, 2016)
- Disable color output if not TTY
- Improve OpenSSL detection

## 1.0.0-beta.3 (Nov 16, 2016)

- No longer requires OpenSSL on MacOS and Windows
- Starting to colorize terminal output where it makes sense
- Linux build more portable (centos 5 glibc and statically linked OpenSSL)
- [run] Removed --meta. Metadata printed to stderr unless --silence
- [cp] Support for data connectors (e.g. dropbox:// and s3://)
- [cp] Smarter handling of destination (aware if destination is file or dir)
- [ls] Aware of terminal width and colors like system ls
- [clone] New command for cloning algorithms
- [runlocal] New command - still experimental
- [serve] New command - still experimental

## 1.0.0-beta.2  (Sept 30, 2015)
- [run] Binary I/O support
- [run] Autodetect input type via -d/-D flags (j/t/b variants exist to explicitly set input type)
- [run] Print API response body without parsing it via --response (includes header) and --response-body
- [run] Set algorithm timeout via --timeout flag
- [run] Print algorithm stdout via the --debug flag (author-only)
- [run] Writing output to a file via -o <outfile>. You can still redirect to a file, but -o frees up stdout for other messages
- [run] Print API alerts to stderr by default, can be silenced with --silence
