# Everarc guide

## Configuration file

Everarc reads configuration from TOML.

- Default path: `everarc.toml` in the current working directory.
- Override path: `everarc --config <PATH> <COMMAND>` or `everarc <COMMAND> --config <PATH>`.
- `everarc check` validates configuration.
- `everarc build` validates configuration and generates HTML.

## Current format

The root TOML table requires a `version` key whose value is an unsigned integer.

```toml
version = 1
```

## Commands

### Check

```sh
everarc check
# checking version 1...
```

### Build

`build` renders an HTML document from the configuration. Its default output
path is `everarc.html` in the current working directory. Use `-o` or
`--output` to choose a different path; an existing output file is overwritten.
Use `-w` or `--watch` to rebuild when the configuration file changes; stop
watch mode with `Ctrl-C`. Watch mode writes the HTML file but does not serve it
or refresh a browser.

```sh
everarc build
# creates everarc.html

everarc build --output site.html
# creates site.html

everarc build --watch
# rebuilds everarc.html after changes to everarc.toml
```
