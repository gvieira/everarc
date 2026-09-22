use super::*;

fn recurring_adjustment_config(event: &str) -> Config {
    let source = format!(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[[conversion_rates]]
from = "EUR"
to = "USD"
rate = "1.1"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = {{ amount = "100", currency = "USD" }}
monthly_withdrawal = {{ amount = "40", currency = "USD" }}

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "USD"
value = "0"

{event}
"#
    );
    toml::from_str(&source).expect("test configuration parses")
}

#[test]
fn formats_and_chains_configuration_read_errors() {
    let error = ConfigError::Read {
        path: PathBuf::from("missing.toml"),
        source: io::Error::new(io::ErrorKind::NotFound, "not found"),
    };

    assert_eq!(
        error.to_string(),
        "failed to read configuration file `missing.toml`: not found"
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn rejects_conversion_rates_configured_in_both_directions() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[conversion_rates]]
from = "BTC"
to = "USD"
rate = "100000"

[[conversion_rates]]
from = "USD"
to = "BTC"
rate = "0.00001"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"
assets = []
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::DuplicateConversionRate { .. })
    ));
}

#[test]
fn rejects_scenario_names_longer_than_32_characters() {
    let mut config: Config = toml::from_str(include_str!("../../everarc.toml")).unwrap();
    config.scenarios[0].name = "A scenario name that exceeds 32 chars".into();

    assert!(matches!(
        config.validate(),
        Err(ConfigError::ScenarioNameTooLong { maximum: 32, .. })
    ));
}

#[test]
fn inherits_omitted_scenario_and_asset_values_from_parent() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-12"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0.03"

[[scenarios.assets]]
id = "brokerage"
name = "Brokerage"
currency = "USD"
annual_expected_return = "0.06"
monthly_contribution = { amount = "1000", currency = "USD" }
monthly_withdrawal = { amount = "200", currency = "USD" }
starts = "2026-03"
ends = "2026-10"

[[scenarios.assets.holdings]]
id = "etf"
name = "ETF"
currency = "USD"
value = "10000"

[[scenarios.milestones]]
id = "brokerage-target"
name = "Brokerage target"
asset_id = "brokerage"
target = "50000"

[[scenarios]]
id = "optimistic"
name = "Optimistic"
extends = "base"

[[scenarios.assets]]
id = "brokerage"
annual_expected_return = "0.08"
"#,
    )
    .unwrap();

    config.validate().unwrap();
    let scenario = &config.scenarios[1];
    let asset = &scenario.assets[0];
    assert_eq!(scenario.annual_inflation, Decimal::new(3, 2));
    assert_eq!(asset.name, "Brokerage");
    assert_eq!(asset.annual_expected_return, Decimal::new(8, 2));
    assert_eq!(asset.monthly_contribution.amount, Decimal::new(1000, 0));
    assert_eq!(
        asset.monthly_withdrawal.as_ref().unwrap().amount,
        Decimal::new(200, 0)
    );
    assert_eq!(asset.holdings.len(), 1);
    assert_eq!(asset.starts, Some("2026-03".parse().unwrap()));
    assert_eq!(asset.ends, Some("2026-10".parse().unwrap()));
    assert_eq!(scenario.milestones.len(), 1);
}

#[test]
fn rejects_multiple_selected_scenarios() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
selected = true
annual_inflation = "0"
assets = []

[[scenarios]]
id = "alternative"
name = "Alternative"
selected = true
annual_inflation = "0"
assets = []
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::MultipleSelectedScenarios)
    ));
}

#[test]
fn rejects_monthly_expected_return_configuration() {
    assert!(
        toml::from_str::<Config>(
            r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"
"#
        )
        .is_err()
    );
}

#[test]
fn rejects_flat_amount_and_currency_for_recurring_flow_events() {
    for (event_type, expected_field) in [
        ("set_monthly_contribution", "monthly_contribution"),
        ("set_monthly_withdrawal", "monthly_withdrawal"),
    ] {
        let source = format!(
            r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = {{ amount = "0", currency = "USD" }}

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "legacy-setting"
name = "Legacy setting"
type = "{event_type}"
date = "2026-01"
asset_id = "cash"
amount = "100"
currency = "USD"
"#
        );

        let error = toml::from_str::<Config>(&source).expect_err("legacy event fields fail");
        assert!(error.to_string().contains(expected_field), "{error}");
    }
}

#[test]
fn rejects_recurring_flow_adjustments_in_a_different_currency() {
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "raise-contribution"
name = "Raise contribution"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { amount = "10", currency = "EUR" }
"#,
    );

    assert!(matches!(
        config.validate(),
        Err(ConfigError::RecurringFlowAdjustmentCurrencyMismatch {
            flow: "contribution",
            ..
        })
    ));
}

#[test]
fn rejects_recurring_flow_adjustments_that_make_the_flow_negative() {
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "reduce-withdrawal"
name = "Reduce withdrawal"
type = "adjust_monthly_withdrawal"
date = "2026-02"
asset_id = "cash"
monthly_withdrawal = { amount = "-50", currency = "USD" }
"#,
    );

    assert!(matches!(
        config.validate(),
        Err(ConfigError::NegativeRecurringFlowAfterAdjustment {
            flow: "withdrawal",
            ..
        })
    ));
}

#[test]
fn rejects_recurring_flow_adjustment_rates_below_negative_one() {
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "over-reduce-contribution"
name = "Over-reduce contribution"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { rate = "-1.01" }
"#,
    );

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidRecurringFlowAdjustmentRate {
            flow: "contribution",
            ..
        })
    ));
}

#[test]
fn rejects_mixed_and_incomplete_recurring_flow_adjustments() {
    for adjustment in [
        r#"{ amount = "10", currency = "USD", rate = "0.1" }"#,
        r#"{ amount = "10" }"#,
        r#"{ rate = "0.1", currency = "USD" }"#,
    ] {
        let source = format!(
            r#"
id = "invalid-adjustment"
name = "Invalid adjustment"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = {adjustment}
"#
        );

        toml::from_str::<Event>(&source).expect_err("invalid adjustment object fails");
    }
}

#[test]
fn rejects_invalid_and_unsupported_event_recurrence() {
    let mut zero_interval = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "zero-interval"
name = "Zero interval"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { rate = "0.1" }
recurrence = { every = 0, unit = "months" }
"#,
    );
    assert!(matches!(
        zero_interval.validate(),
        Err(ConfigError::InvalidEventRecurrence { .. })
    ));

    let mut unsupported = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "recurring-setter"
name = "Recurring setter"
type = "set_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { amount = "200", currency = "USD" }
recurrence = { every = 1, unit = "years" }
"#,
    );
    assert!(matches!(
        unsupported.validate(),
        Err(ConfigError::UnsupportedEventRecurrence { .. })
    ));

    let mut ends_before_start = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "backwards-recurrence"
name = "Backwards recurrence"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { rate = "0.1" }
recurrence = { every = 1, unit = "months", until = "2026-01" }
"#,
    );
    assert!(matches!(
        ends_before_start.validate(),
        Err(ConfigError::EventRecurrenceEndsBeforeStart { .. })
    ));
}

#[test]
fn rejects_a_later_recurring_adjustment_that_makes_the_flow_negative() {
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "recurring-reduction"
name = "Recurring reduction"
type = "adjust_monthly_withdrawal"
date = "2026-02"
asset_id = "cash"
monthly_withdrawal = { amount = "-30", currency = "USD" }
recurrence = { every = 1, unit = "months" }
"#,
    );

    assert!(matches!(
        config.validate(),
        Err(ConfigError::NegativeRecurringFlowAfterAdjustment {
            flow: "withdrawal",
            ..
        })
    ));
}

#[test]
fn rejects_contributions_in_unrelated_currencies() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[conversion_rates]]
from = "BTC"
to = "USD"
rate = "10000"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "bitcoin"
name = "Bitcoin"
currency = "BTC"
annual_expected_return = "0"
monthly_contribution = { amount = "100", currency = "BRL" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "BTC"
value = "0"
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidContributionCurrency { .. })
    ));
}

#[test]
fn rejects_withdrawals_in_unrelated_currencies() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }
monthly_withdrawal = { amount = "100", currency = "BRL" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidWithdrawalCurrency { .. })
    ));
}

#[test]
fn rejects_living_cost_inflation_of_negative_one_or_less() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[future_living_costs]]
id = "housing"
name = "Housing"
annual_inflation = "-1"
monthly_cost = "100"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidFutureLivingCostInflation { .. })
    ));
}

#[test]
fn rejects_annual_inflation_of_negative_one_or_less() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "-1"
assets = []
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidAnnualInflation { .. })
    ));
}
