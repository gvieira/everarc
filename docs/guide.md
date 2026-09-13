# Everarc guide

## Configuration file

Everarc reads configuration from TOML.

- Default path: `everarc.toml` in the current working directory.
- Override path: `everarc --config <PATH> <COMMAND>` or `everarc <COMMAND> --config <PATH>`.
- `everarc check` validates configuration.
- `everarc build` validates configuration and generates HTML.

## Current format

The root TOML table requires a `version` key whose value is an unsigned integer,
a single `plan` table, and at least one scenario.

```toml
version = 1

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
monthly_cost = "1200.00"

# Every config has one or more scenarios.
[[scenarios]]
id = "baseline"
name = "Baseline"
# Annual decimal fraction: 0.03 means 3%; negative values represent deflation.
annual_inflation = "0.03"

# Every scenario has one or more assets.
[[scenarios.assets]]
id = "brokerage"
name = "Brokerage"
currency = "USD"
initial_value = "25000.00"
# Decimal fraction: 0.5 means a 50% expected monthly return.
monthly_expected_return = "0.005"
monthly_contribution = "1000.00"

# Optional events apply in their declaration order.
[[scenarios.events]]
id = "car-purchase"
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
monthly_expected_return = "0.005"
monthly_contribution = "1000.00"
```

Each conversion rate must connect exactly one currency to `plan.currency`; for
a USD plan, `BTC` → `USD` is valid but `BTC` → `BRL` is not. `rate` is a
positive quoted decimal, and its direction is `1 from = rate to`. Rates are
optional when all values use the plan currency. Duplicate `from`/`to` pairs
are not allowed.

Every scenario needs a nonblank `id` and `name`, plus an `annual_inflation`
quoted decimal. It is an annual fraction: `"0.03"` means 3%, while `"-0.01"`
means 1% annual deflation. Scenario IDs must be unique. `extends` is optional and refers to another scenario ID; it may point forward
or backward in the file. Everarc rejects unknown parent IDs, self-extension,
and extension cycles. Inheritance is preserved for later runtime expansion; it
is not copied or merged while the config loads.

Every scenario also needs one or more `[[scenarios.assets]]` tables. Asset IDs
and names must be nonblank; IDs are unique within their scenario. `currency`
is a required opaque identifier, so it may be `USD`, `BTC`, or another
consistently used currency. The three financial fields are nonnegative quoted
decimals. `monthly_expected_return` is a fraction: `"0.5"` means 50% per
month, while `"0.005"` means 0.5%. Asset-currency conversion validation comes
later.

Events are optional and are processed in TOML declaration order. Every event
has a nonblank ID unique within its scenario, a `YYYY-MM` date within the plan
range, and an `asset_id`. The asset may be local to the scenario or inherited
from a parent scenario. `asset_adjustment` uses a signed `amount` in the
asset's currency. `set_monthly_contribution` uses a nonnegative `amount`, and
`set_monthly_expected_return` uses a nonnegative decimal-fraction `rate`;
both define values for later projection work. Inflation-changing events are not
supported yet.

Milestones are optional balance thresholds with no date. Root `[[milestones]]`
are total-balance goals shared by every scenario and use `plan.currency`.
`[[scenarios.milestones]]` are asset-balance goals; they use their target
asset's currency and may target inherited assets. IDs and names must be
nonblank, IDs are unique within their own scope, and all `target` values are
strictly positive quoted decimals.

`[[future_living_costs]]` is an optional collection of expected monthly living
costs at the plan's end, expressed in today's `plan.currency` purchasing power.
It is not a record of current spending. Each cost has a nonblank, unique `id`,
a nonblank `name`, and a strictly positive quoted `monthly_cost`. Scenario
inflation will convert these reference values to future nominal money in later
projection work.

## Commands

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
