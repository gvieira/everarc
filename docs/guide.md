# Everarc guide

## Configuration file

Everarc reads configuration from TOML.

- Default path: `everarc.toml` in the current working directory.
- Override path: `everarc --config <PATH> <COMMAND>` or `everarc <COMMAND> --config <PATH>`.
- `everarc check` validates configuration.
- `everarc build` validates configuration and generates HTML.

## Current format

The root TOML table requires a single `plan` table and at least one scenario.

```toml
# Required dashboard language and number-formatting conventions.
[display]
locale = "pt-BR"

[plan]
# Reporting currency. Identifiers are compared exactly, so this can be USD, BRL,
# BTC, or another currency identifier used consistently throughout the config.
currency = "USD"

# First and last included months, using the exact YYYY-MM format.
# `end` may equal `start`, but cannot be before it.
start = "2026-01"
end = "2030-12"

# Optional static rates. One BTC equals 67,500.25 USD.
[[conversion_rates]]
from = "BTC"
to = "USD"
rate = "67500.25"

# Optional total-balance goals shared by every scenario, in the plan currency.
[[milestones]]
id = "financial-independence"
name = "Financial independence"
target = "1000000.00"

# Optional end-of-plan monthly cost estimates, in today's plan-currency money.
[[future_living_costs]]
id = "health"
name = "Health insurance"
# Optional long-form detail shown when hovering over the dashboard row.
description = "Private coverage and routine out-of-pocket care."
# Optional effective annual override; otherwise the scenario's rate applies.
annual_inflation = "0.04"
monthly_cost = "1200.00"

# Every config has one or more scenarios.
[[scenarios]]
id = "baseline"
name = "Baseline"
# Optional detail shown in the scenario selector and plan summary.
description = "Expected returns and contributions under current assumptions."
# Optional: marks the scenario you intend to follow.
selected = true
# Annual decimal fraction: 0.03 means 3%; negative values represent deflation.
annual_inflation = "0.03"

# Every scenario has one or more assets.
[[scenarios.assets]]
id = "brokerage"
name = "Brokerage"
currency = "USD"
initial_value = "25000.00"
# Effective annual decimal fraction: 0.06 means a 6% expected annual return.
annual_expected_return = "0.06"
monthly_contribution = { amount = "1000.00", currency = "USD" }

# Optional events apply in their declaration order.
[[scenarios.events]]
id = "car-purchase"
name = "Car purchase"
date = "2027-06"
type = "asset_adjustment"
asset_id = "brokerage"
# Signed amount in the asset currency; negative withdraws funds.
amount = "-25000.00"

# Optional asset-balance targets use the target asset's currency.
[[scenarios.milestones]]
id = "brokerage-100k"
name = "Brokerage reaches 100k"
asset_id = "brokerage"
target = "100000.00"

# A scenario may extend any scenario declared in this config.
[[scenarios]]
id = "optimistic"
name = "Optimistic"
annual_inflation = "0.02"
extends = "baseline"

[[scenarios.assets]]
id = "brokerage"
name = "Brokerage"
currency = "USD"
initial_value = "25000.00"
annual_expected_return = "0.06"
monthly_contribution = { amount = "1000.00", currency = "USD" }
```

`[display]` is required. `locale` controls generated-dashboard text and number
formatting; supported values are `en-US` and `pt-BR`. It does not change CLI or
configuration-check messages.

A scenario may optionally set `selected = true` to identify the one you intend
to follow. At most one scenario may be selected. The dashboard opens on it and
marks it in the scenario bar; when none is selected, it opens on the first
scenario.

Each conversion rate must connect exactly one currency to `plan.currency`; for
a USD plan, `BTC` → `USD` is valid but `BTC` → `BRL` is not. `rate` is a
positive quoted decimal, and its direction is `1 from = rate to`. Rates are
optional when all assets use the plan currency. Every asset in another currency
needs a rate connecting it to `plan.currency`; Everarc uses a direct
asset-currency → plan-currency rate when available, or reciprocates a reverse
plan-currency → asset-currency rate. Only one rate may connect a given
non-plan currency to the plan currency: configuring both directions (or the
same direction twice) is not allowed.

Every scenario needs a nonblank `id` and `name`, plus an `annual_inflation`
quoted decimal greater than `"-1"`. An optional `description` appears in the
dashboard's scenario selector and plan summary. Inflation is an annual fraction:
`"0.03"` means
3%, while `"-0.01"` means 1% annual deflation. Everarc uses this rate to
convert future living costs from plan-start purchasing power to end-of-plan
nominal money, applying it across every inclusive plan month. Scenario IDs
must be unique. `extends` is optional and refers to another scenario ID; it may point forward
or backward in the file. Everarc rejects unknown parent IDs, self-extension,
and extension cycles. Inheritance is preserved for later runtime expansion; it
is not copied or merged while the config loads.

Every scenario also needs one or more `[[scenarios.assets]]` tables. Asset IDs
and names must be nonblank; IDs are unique within their scenario. `currency`
is a required opaque identifier, so it may be `USD`, `BTC`, or another
consistently used currency. `initial_value` is a nonnegative quoted decimal.
`annual_expected_return` is an effective annual fraction: `"0.5"` means 50%
per year, while `"0.06"` means 6%. Everarc compounds it monthly so twelve
projected months produce the configured annual return. `monthly_contribution`
is an object with a nonnegative quoted-decimal `amount` and a `currency` equal
to either the asset currency or `plan.currency`. Plan-currency contributions
are converted to the asset currency using the static configured rate before
being added to the balance. Assets outside `plan.currency` require a usable
conversion rate.

Events are optional and are processed in TOML declaration order. Every event
has a nonblank ID unique within its scenario, a nonblank human-facing name, a
`YYYY-MM` date within the plan range, and an `asset_id`. The asset may be local to the scenario or inherited
from a parent scenario. `asset_adjustment` uses a signed `amount` in the
asset's currency. `set_monthly_contribution` uses a nonnegative `amount` and
a required `currency` equal to either the asset currency or `plan.currency`;
it takes effect for that month's contribution and every following month. Where
multiple settings take effect in the same month, the last declaration wins.
`set_annual_expected_return` uses a nonnegative effective annual
decimal-fraction `rate` and takes effect for that month's return and every
following month; multiple same-month settings use the last declaration.
Inflation-changing events are not supported yet.

Milestones are optional balance thresholds with no date. Root `[[milestones]]`
are total-balance goals shared by every scenario and use `plan.currency`.
`[[scenarios.milestones]]` are asset-balance goals; they use their target
asset's currency and may target inherited assets. IDs and names must be
nonblank, IDs are unique within their own scope, and all `target` values are
strictly positive quoted decimals.

`[[future_living_costs]]` is an optional collection of expected monthly living
costs at the plan's end, expressed in today's `plan.currency` purchasing power.
It is not a record of current spending. Each cost has a nonblank, unique `id`,
a nonblank `name`, an optional `description`, and a strictly positive quoted
`monthly_cost`. An optional effective annual `annual_inflation` overrides the
scenario's rate for that cost and must be greater than `"-1"`; when omitted,
the scenario rate applies. The dashboard shows descriptions and each cost's
resolved annual rate in hover tooltips, plus each cost's share of the current
monthly-total view. Projections convert
these reference values to end-of-plan nominal money using each cost's resolved
annual inflation. The dashboard shows these adjusted end-of-plan nominal values
by default; its inflation switch can show the configured today-money values for
comparison.

## Commands

### Version

```sh
everarc --version
```

### Check

```sh
everarc check
# configuration is valid.
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
