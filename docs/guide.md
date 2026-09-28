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
# Optional annual retirement withdrawal rate; defaults to 4%.
withdrawal_rate = "0.04"

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
# Scenario names are limited to 32 characters for the dashboard picker.
name = "Baseline"
# Optional detail shown in the scenario selector and plan summary.
description = "Expected returns and contributions under current assumptions."
# Optional: marks the scenario you intend to follow.
selected = true
# Annual decimal fraction: 0.03 means 3%; negative values represent deflation.
annual_inflation = "0.03"
# Optional positive monthly income with an explicit currency.
monthly_income = { amount = "5000.00", currency = "USD" }

# Root scenarios have one or more assets.
[[scenarios.assets]]
id = "brokerage"
name = "Brokerage"
currency = "USD"
# Effective annual decimal fraction: 0.06 means a 6% expected annual return.
annual_expected_return = "0.06"
monthly_contribution = { amount = "1000.00", currency = "USD" }
monthly_withdrawal = { amount = "250.00", currency = "USD" }

[[scenarios.assets.holdings]]
id = "world-etf"
name = "World ETF"
currency = "USD"
value = "25000.00"

# Optional events apply in their declaration order.
[[scenarios.events]]
id = "car-purchase"
name = "Car purchase"
date = "2027-06"
type = "asset_adjustment"
asset_id = "brokerage"
# Signed amount in the asset currency; negative withdraws funds.
amount = "-25000.00"
# Repeat every five years, stopping when the plan or asset ends.
recurrence = { every = 60, unit = "months", until = "2042-06" }

[[scenarios.events]]
id = "retirement-withdrawal"
name = "Start retirement withdrawals"
date = "2040-01"
type = "set_monthly_withdrawal"
asset_id = "brokerage"
monthly_withdrawal = { amount = "3000.00", currency = "USD" }

[[scenarios.events]]
id = "salary-increase"
name = "Invest salary increase"
date = "2032-04"
type = "adjust_monthly_contribution"
asset_id = "brokerage"
# Change the active contribution by 5%; negative rates reduce it.
monthly_contribution = { rate = "0.05" }
recurrence = { every = 1, unit = "years" }

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
extends = "baseline"

# Omitted fields inherit from the same-ID parent asset and scenario.
[[scenarios.assets]]
id = "brokerage"
annual_expected_return = "0.07"
```

`[display]` is required. `locale` controls generated-dashboard text and number
formatting; supported values are `en-US` and `pt-BR`. It does not change CLI or
configuration-check messages.

A scenario may optionally set `selected = true` to identify the one you intend
to follow. At most one scenario may be selected. The dashboard opens on it and
marks it in the scenario bar; when none is selected, it opens on the first
scenario.

The generated dashboard's **Appearance** selector in the footer defaults to **Auto**, following
your browser's light/dark setting, including changes while the dashboard is open.
Choose **Light** or **Dark** to override it; your choice is saved in this browser
when storage is available. Choose **Auto** again to clear the saved override.

## Actual balances

Optional actual end-of-month balances are shared across scenarios and grouped by
month. The quoted month key uses `YYYY-MM`; each asset ID maps to a quoted
decimal balance in that asset's own currency.

```toml
[actual_balances."2026-02"]
brokerage = "28450.75"
crypto = "0.12"

[actual_balances."2026-03"]
brokerage = "29110.40"
```

Missing asset/month entries are unknown, never zero. Balances may be negative.
An actual-balance month must be within the plan range and every recorded asset
ID must occur in at least one scenario. When a recorded ID occurs in multiple
scenarios, its asset currency must be the same in each. Start recording with
the last completed calendar month; Everarc does not require records for every
asset or month.

Each conversion rate must connect exactly one currency to `plan.currency`; for
a USD plan, `BTC` → `USD` is valid but `BTC` → `BRL` is not. `rate` is a
positive quoted decimal, and its direction is `1 from = rate to`. Rates are
optional when all assets and holdings use the plan currency. Every asset or
holding in another currency needs a rate connecting it to `plan.currency`; Everarc uses a direct
asset-currency → plan-currency rate when available, or reciprocates a reverse
plan-currency → asset-currency rate. Only one rate may connect a given
non-plan currency to the plan currency: configuring both directions (or the
same direction twice) is not allowed.

A holding may use a different currency from its parent asset. For example, a
BRL plan can value a BTC holding inside a USD asset using each currency's rate
to BRL:

```toml
[[conversion_rates]]
from = "USD"
to = "BRL"
rate = "5.00"

[[conversion_rates]]
from = "BTC"
to = "BRL"
rate = "500000.00"

[[scenarios.assets]]
id = "crypto"
name = "Crypto account"
currency = "USD"
annual_expected_return = "0.08"
monthly_contribution = { amount = "100", currency = "USD" }

[[scenarios.assets.holdings]]
id = "bitcoin"
name = "Bitcoin"
currency = "BTC"
value = "0.10"
```

Everarc converts the holding from BTC to BRL, then from BRL to USD, to derive
the asset's initial value.

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

Each root scenario needs one or more `[[scenarios.assets]]` tables. An extending
scenario may omit assets entirely or declare only the fields it overrides on a
same-ID inherited asset. Asset IDs
and names must be nonblank; IDs are unique within their scenario. `currency`
is a required opaque identifier, so it may be `USD`, `BTC`, or another
consistently used currency. Each asset must define one or more nested
`[[scenarios.assets.holdings]]` tables. Holdings have nonblank, unique IDs and
names within their asset, an optional `description`, a currency, and a nonnegative quoted-decimal `value`.
Their values are converted into the asset currency and summed to derive the
asset's initial value. A holding may use a different currency; Everarc converts
it through `plan.currency` using the configured static rates. In an extending
scenario, an asset with an inherited ID overrides only its declared fields;
omitted fields, including holdings, inherit from the parent asset. Optional
inclusive `starts` and `ends` `YYYY-MM` fields default to the plan start and
end. Holdings are the asset's opening balance in `starts`, so a manual split
can end one asset and start separately configured assets in the same month.
For example, an asset ending at a manual split and each replacement asset can
use `ends = "2032-01"` and `starts = "2032-01"`, respectively.
`annual_expected_return` is an effective annual fraction: `"0.5"` means 50%
per year, while `"0.06"` means 6%. Everarc compounds it monthly so twelve
projected months produce the configured annual return. An optional scenario
`monthly_income` is an optional object with a positive quoted-decimal `amount`
and a `currency` that is either `plan.currency` or has a configured conversion
rate to it; child scenarios inherit it unless they specify their own value.
`monthly_contribution`
and the optional `monthly_withdrawal` are objects with nonnegative
quoted-decimal `amount` values and a `currency` equal to either the asset
currency or `plan.currency`. Contributions and withdrawals are independent and
may occur in the same month; Everarc adds the gross contribution and subtracts
the gross withdrawal. Plan-currency flows are converted to the asset currency
using the static configured rate. Omitting `monthly_withdrawal` is equivalent
to a zero withdrawal in the asset currency. Assets outside `plan.currency`
require a usable conversion rate.

Events are optional and are processed in TOML declaration order. Every event
has a nonblank ID unique within its scenario, a nonblank human-facing name, a
`YYYY-MM` date within the plan range, and an `asset_id`. The asset may be local to the scenario or inherited
from a parent scenario. `asset_adjustment` uses a signed `amount` in the
asset's currency. `set_monthly_contribution` and `set_monthly_withdrawal` use
respective `monthly_contribution = { amount = "...", currency = "..." }` and
`monthly_withdrawal = { amount = "...", currency = "..." }` objects with a
nonnegative `amount` and a required `currency` equal to either the asset currency or
`plan.currency`. `adjust_monthly_contribution` and `adjust_monthly_withdrawal`
accept either a fixed signed money object, such as
`monthly_contribution = { amount = "250.00", currency = "USD" }`, or a
percentage object, such as `monthly_contribution = { rate = "0.05" }`. Positive
values increase and negative values reduce the active recurring flow. Fixed
adjustments must use the active setting's currency and must not make the result
negative. Percentage adjustments inherit that currency and calculate
`active amount * (1 + rate)`, so successive adjustments compound. A rate of
`-1` sets the flow to zero; lower rates are invalid. Setters remain absolute
while adjustments are relative. Each independently takes effect for that month's flow and
every following month. Recurring-flow setters and adjustments in the same month
apply in declaration order. `asset_adjustment`, `adjust_monthly_contribution`,
and `adjust_monthly_withdrawal` may include
`recurrence = { every = 1, unit = "years" }` or use `unit = "months"` with
any positive integer interval. The event's `date` is its first occurrence. An
optional `until = "YYYY-MM"` stops recurrence at an inclusive event-specific
month and must not precede the first occurrence. It does not need to align with
the interval. Occurrences stop automatically after the earliest of `until`, the
plan end, or the target asset's inclusive `ends` month; an occurrence exactly in
that ending month still applies. An `until` beyond the plan or asset end is valid
but does not extend those existing limits. Percentage adjustments compound at each occurrence. Recurring signed
asset adjustments remain uncapped and may produce a negative balance. Recurring
occurrences preserve parent-before-child and TOML declaration order when they
share a month. Absolute setters and expected-return setters do not support
recurrence. `set_annual_expected_return` uses a
nonnegative effective annual
decimal-fraction `rate` and takes effect for that month's return and every
following month; multiple same-month settings use the last declaration.
Inflation-changing events are not supported yet.

Milestones are optional balance thresholds with no date. Root `[[milestones]]`
are total-balance goals shared by every scenario and use `plan.currency`.
`[[scenarios.milestones]]` are asset-balance goals; they use their target
asset's currency and may target inherited assets. Child scenarios inherit
parent asset milestones in addition to their own. IDs and names must be
nonblank, IDs are unique within their resolved scenario, and all `target` values are
strictly positive quoted decimals.

The dashboard chart has `↤` to focus its first three years, `+` and `−` to
change its visible month range, and `⛶` to restore the full plan. When zoomed,
drag across the plot to pan. Switching scenarios preserves the selected month
and visible date range. The chart recalculates its Y-axis from visible balances,
so shorter periods use their available vertical space; milestones
above that visible range are hidden. A compact cash-flow chart below the balance
plot shows gross recurring contributions above zero and gross recurring withdrawals
below zero, summed across active assets and converted to `plan.currency`. Each
stack uses the matching asset colors to show how the flow is divided. The visible
inflow and outflow bounds are rounded independently, while sharing one linear scale;
the zero baseline moves proportionally so magnitudes remain comparable without
wasting chart height. Returns and
`asset_adjustment` events are excluded. Its hover panel shows total contributions,
the net invested percentage of monthly income when configured, total withdrawals,
signed net flow, and each asset's absolute amount and share of its respective
flow. The cash-flow chart shares the balance chart's month selection,
pin, zoom, pan, keyboard navigation, and scenario; hiding asset lines does not
change its totals. The balance chart's compact hover panel shows the current month's total
balance, passive income, gross contribution, and gross withdrawal. When a
scenario has `monthly_income`, the contribution value also shows the net
recurring investment (contributions minus withdrawals) as a percentage of that
income. A persistent inspector below the
charts follows the selected month and shows each
active asset's balance, return, contribution, withdrawal, passive income,
lifecycle changes, and that month's events. Click a month to pin its cursor and inspectors while moving
the pointer; click that month again to unpin it, or click another month to move
the pin. When pinned, arrow buttons move to the previous or next notable month:
a month containing an asset event or an asset starting or ending. Keyboard month
navigation also pins its selection. Dragging a zoomed
chart pans without changing the pin. Asset starts and ends appear as upward and
downward triangle markers; hovering or focusing one identifies the asset and
change. Each asset card's eye button shows or hides that asset's chart line,
event markers, lifecycle markers, and milestones; the card stays visible
and dimmed so the asset can be restored. The inspector also provides show-all
and hide-all actions. Visibility follows same-ID assets across scenario switches
but never changes projected balances or total net worth. The passive-income
summary includes its annualized yield (monthly passive income multiplied by
twelve, divided by total balance). `plan.withdrawal_rate` is an optional annual
decimal rate greater than zero and no more than one; it defaults to `"0.04"`.
It is used only for the dashboard's end-of-plan retirement-withdrawal estimate
and does not alter projections or configured withdrawals. **Resumo do plano**
defaults to monthly passive income; its display switch can show that retirement
withdrawal estimate instead with the **Safe withdrawal rate** switch.

Recorded actual balances are shared across scenarios and are end-of-month
checkpoints: an observed asset balance replaces its projection for that month,
and its projection resumes from the real balance in the following month. The
projection chart shows one translucent actual-total area; it has a value only
for months where every asset active in the selected scenario has a recorded
balance, so partial months produce gaps rather than misleading totals. The
selected-month inspector shows each active asset's projected balance, recorded
actual balance, and signed actual-minus-projected difference; missing records
appear as an em dash when other assets have observations for the selected month.
Foreign-currency actuals use the configured static conversion rates for total
comparison.

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
Use `--data-output PATH` to also write a pretty-printed JSON projection for
LLMs and other tools. It contains calculated monthly and per-asset balances,
cash flow, passive income, applied events, actual-balance comparisons, future
living-cost projections, and goal results for every scenario. Decimal values
are JSON strings to preserve their exact precision. It does not include
localized dashboard text, chart geometry, browser state, or configuration
values that are not needed to understand the calculated results. Use `-w` or
`--watch` to rebuild when the configuration file
changes; stop watch mode with `Ctrl-C`. Watch mode updates both output files,
but does not serve them or refresh a browser.

```sh
everarc build
# creates everarc.html

everarc build --output site.html
# creates site.html

# creates everarc.html and dashboard-data.json
everarc build --data-output dashboard-data.json

everarc build --watch
# rebuilds everarc.html after changes to everarc.toml
```
