## 1.0.0-beta.2
- [run] Binary I/O support
- [run] Autodetect input type via -d/-D flags (j/t/b variants exist to explicitly set input type)
- [run] Print API response body without parsing it via --response (includes header) and --response-body
- [run] Set algorithm timeout via --timeout flag
- [run] Print algorithm stdout via the --debug flag (author-only)
- [run] Writing output to a file via -o <outfile>. You can still redirect to a file, but -o frees up stdout for other messages
- [run] Print API alerts to stderr by default, can be silenced with --silence
