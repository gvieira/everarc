# Contributing to Everarc

Bug reports, documentation improvements, and focused pull requests are welcome.
For substantial features or design changes, open an issue first to agree on
scope before implementing them.

## Development

Install [Rust](https://rustup.rs/) 1.87 or newer, then run these commands from
the checkout:

```sh
cargo build --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

GitHub Actions runs tests and strict Clippy on clean Linux, macOS, and Windows
checkouts. It also checks formatting and tests with Rust 1.87.0 on Linux. To
check minimum-version compatibility locally, install that toolchain and run
`cargo +1.87.0 test --locked`.

Use `cargo fmt` to apply formatting. Clippy warnings fail CI; resolve them before
submitting a pull request. Tests use the synthetic configuration in
`tests/fixtures/everarc.toml`; no personal root configuration is required.

The test suite includes a CLI watcher smoke test that checks HTML and JSON
rebuilds after a configuration edit. It uses temporary synthetic data and runs
in the CI OS matrix. Run it alone with `cargo test --locked --test watch`.

For a CLI smoke test:

```sh
cargo run --locked -- --config tests/fixtures/everarc.toml check
cargo run --locked -- --config tests/fixtures/everarc.toml build
```

The build writes `everarc.html` in the current directory, overwriting an
existing file at that path. Open it in a browser to check dashboard changes.

## Pull requests

- Keep changes small and focused; explain the problem and the approach.
- Add regression tests for fixes and tests for new behavior, especially changes
  to financial calculations.
- Update `docs/guide.md` when changing CLI behavior or configuration. The guide
  is embedded in the binary by `everarc guide`.
- For dashboard changes, check affected interactions and both appearance modes;
  check both supported locales when changing displayed text or formatting.
- Describe how you tested the change and any remaining limitations.
- Use synthetic data in examples, fixtures, screenshots, and attachments.
  Do not commit personal financial configurations or generated reports.

## Bug reports

Search existing [GitHub Issues](https://github.com/gvieira/everarc/issues) before
opening a new report. Include:

- Everarc version (`everarc --version`), operating system, and browser version
  for dashboard problems.
- Steps to reproduce, expected behavior, and actual behavior or error output.
- A minimal synthetic TOML configuration when relevant.

HTML and JSON exports can contain balances and monthly notes. Do not attach
personal reports or credentials to public issues.

Report security vulnerabilities privately using [SECURITY.md](SECURITY.md),
not a public issue.
