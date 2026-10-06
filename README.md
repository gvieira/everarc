# Everarc

Everarc is a command-line tool for long-term financial planning. You describe
assets, contributions, withdrawals, and expected returns in a TOML file; it
turns them into an interactive HTML dashboard.

Use it to compare scenarios, plan for large expenses, and see how far your
savings might go in retirement. You can record actual monthly balances as you
go, so future projections start from what you have rather than what you hoped
to have.

The numbers depend on your assumptions. They aren't predictions.

## Features

- Compare scenarios without duplicating the whole plan.
- Schedule contributions, withdrawals, and one-off or recurring events.
- Track actual balances alongside projections.
- Model multiple currencies, balance goals, and retirement living costs.

## Getting started

You'll need [Rust](https://rustup.rs/) to build from source. From this checkout:

```sh
cargo install --path .
```

Create an `everarc.toml` using the example in the [guide](docs/guide.md), then run:

```sh
everarc check
everarc build
```

Open `everarc.html` in your browser to explore the results. To rebuild whenever
you edit the configuration:

```sh
everarc build --watch
```

You can also export the projections as JSON for other tools:

```sh
everarc build --data-output everarc.json
```

The [guide](docs/guide.md) covers configuration, scenario inheritance, events,
currencies, and the dashboard. It's also available locally with `everarc guide`.

## Working with LLMs

Everarc is built to be useful to LLMs, too. The configuration is plain text,
and the JSON export gives them calculated results to work with. An LLM can help
adjust assumptions or interpret projections; Everarc does the calculations.

## How it works

Everarc is written in Rust. It reads and validates the TOML, calculates
projections month by month, and renders an HTML dashboard you can open in your
browser. There's no server to run.
