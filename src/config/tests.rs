use super::*;

#[test]
fn guide_getting_started_example_validates_and_projects() {
    let guide = include_str!("../../docs/guide.md");
    let (_, example) = guide
        .split_once("```toml\n")
        .expect("guide contains a TOML example");
    let (example, _) = example
        .split_once("```")
        .expect("guide TOML example has a closing fence");
    let mut config: Config = toml::from_str(example).expect("guide example parses");
    config.validate().expect("guide example validates");

    let projection = crate::projection::PlanProjection::from(&config);
    assert_eq!(projection.scenarios.len(), 2);
    for scenario in &projection.scenarios {
        assert_eq!(
            scenario.total_net_worth.len(),
            usize::try_from(config.plan.inclusive_month_count()).expect("month count fits usize")
        );
    }
}

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
fn defaults_plan_withdrawal_rate_to_four_percent() {
    assert_eq!(
        recurring_adjustment_config("").plan.withdrawal_rate,
        Decimal::new(4, 2)
    );
}

#[test]
fn rejects_invalid_plan_withdrawal_rates() {
    for rate in [Decimal::ZERO, Decimal::new(101, 2)] {
        let mut config = recurring_adjustment_config("");
        config.plan.withdrawal_rate = rate;

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidWithdrawalRate { rate: invalid_rate }) if invalid_rate == rate
        ));
    }
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
    let mut config: Config =
        toml::from_str(include_str!("../../tests/fixtures/everarc.toml")).unwrap();
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
monthly_income = { amount = "5000", currency = "USD" }

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
    assert_eq!(
        scenario.monthly_income.as_ref().map(|income| income.amount),
        Some(Decimal::new(5000, 0))
    );
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
fn rejects_nonpositive_monthly_income() {
    for income in [Decimal::ZERO, Decimal::NEGATIVE_ONE] {
        let mut config: Config =
            toml::from_str(include_str!("../../tests/fixtures/everarc.toml")).unwrap();
        config.scenarios[0].monthly_income = Some(MonthlyIncome {
            amount: income,
            currency: Currency("USD".into()),
        });

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidMonthlyIncome { .. })
        ));
    }
}

#[test]
fn rejects_monthly_income_without_a_plan_currency_conversion() {
    let mut config: Config =
        toml::from_str(include_str!("../../tests/fixtures/everarc.toml")).unwrap();
    config.scenarios[0]
        .monthly_income
        .as_mut()
        .unwrap()
        .currency = Currency("EUR".into());

    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidMonthlyIncomeCurrency { .. })
    ));
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

        let mut config: Config = toml::from_str(&source).expect("event declarations parse");
        let error = config.validate().expect_err("legacy event fields fail");
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

#[test]
fn accepts_partial_actual_balances_in_asset_currency() {
    let mut config: Config = toml::from_str(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[actual_months."2026-02"]
note = "An unexpected expense."
balances = { cash = "-12.50" }

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
id = "opening"
name = "Opening"
currency = "USD"
value = "0"
"#,
    )
    .expect("configuration parses");

    config.validate().unwrap();
    assert_eq!(
        config.actual_months[&"2026-02".parse().unwrap()].balances["cash"].0,
        Decimal::new(-1250, 2)
    );
    assert_eq!(
        config.actual_months[&"2026-02".parse().unwrap()]
            .note
            .as_deref(),
        Some("An unexpected expense.")
    );
}

#[test]
fn validates_note_only_actual_months_and_rejects_old_or_unknown_fields() {
    let source = r#"
[display]
locale = "en-US"
[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[actual_months."2026-02"]
note = "Saved more than expected."

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"
[[scenarios.assets]]
id = "note"
name = "An asset named note"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }
[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "USD"
value = "0"
"#;
    let mut config: Config = toml::from_str(source).unwrap();
    config.validate().unwrap();
    let actual = &config.actual_months[&"2026-02".parse().unwrap()];
    assert!(actual.balances.is_empty());
    assert_eq!(actual.note.as_deref(), Some("Saved more than expected."));

    let mut outside: Config = toml::from_str(&source.replace("2026-02", "2025-12")).unwrap();
    assert!(matches!(
        outside.validate(),
        Err(ConfigError::ActualMonthOutsidePlan { .. })
    ));

    for invalid in [
        source.replace("actual_months", "actual_balances"),
        source.replace("note =", "notes ="),
        source.replace("note = \"Saved more than expected.\"", "note = 123"),
    ] {
        assert!(toml::from_str::<Config>(&invalid).is_err());
    }

    let with_balance = source.replace(
        "note = \"Saved more than expected.\"",
        "balances = { note = \"12.50\" }",
    );
    let mut config: Config = toml::from_str(&with_balance).unwrap();
    config.validate().unwrap();
    let actual = &config.actual_months[&"2026-02".parse().unwrap()];
    assert!(actual.note.is_none());
    assert_eq!(actual.balances["note"].0, Decimal::new(1250, 2));
}

#[test]
fn rejects_actual_balances_outside_the_plan_or_for_unknown_assets() {
    let outside_plan = r#"
[actual_months."2025-12"]
balances = { cash = "1" }
"#;
    let unknown_asset = r#"
[actual_months."2026-02"]
balances = { other = "1" }
"#;

    for (record, expected) in [(outside_plan, "outside"), (unknown_asset, "unknown")] {
        let source = format!(
            r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

{record}

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
id = "opening"
name = "Opening"
currency = "USD"
value = "0"
"#
        );
        let mut config: Config = toml::from_str(&source).expect("configuration parses");
        assert!(match expected {
            "outside" => matches!(
                config.validate(),
                Err(ConfigError::ActualMonthOutsidePlan { .. })
            ),
            _ => matches!(
                config.validate(),
                Err(ConfigError::UnknownActualBalanceAsset { .. })
            ),
        });
    }
}

#[test]
fn rejects_actual_balance_asset_currency_mismatches_and_unquoted_values() {
    let mut config: Config = toml::from_str(
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

[actual_months."2026-02"]
balances = { cash = "1" }

[[scenarios]]
id = "usd"
name = "USD"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "USD"
value = "0"

[[scenarios]]
id = "eur"
name = "EUR"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "EUR"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "EUR" }

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "EUR"
value = "0"
"#,
    )
    .expect("configuration parses");
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InconsistentActualBalanceAssetCurrency { .. })
    ));

    let error = toml::from_str::<Config>(
        r#"
[display]
locale = "en-US"
[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"
[actual_months."2026-01"]
balances = { cash = 1 }
scenarios = []
"#,
    )
    .unwrap_err();
    assert!(error.to_string().contains("invalid type"), "{error}");
}

fn event_override_config(overrides: &str) -> Config {
    recurring_adjustment_config(&format!(
        r#"
[[scenarios.events]]
id = "deposit"
name = "Deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "50"

[[scenarios]]
id = "child"
name = "Child"
extends = "base"

{overrides}
"#
    ))
}

#[test]
fn disables_an_inherited_event_without_changing_the_parent() {
    let mut config = event_override_config(
        r#"
[[scenarios.events]]
id = "deposit"
enabled = false
"#,
    );
    config.validate().unwrap();
    let parent = &config.scenarios[0];
    let child = &config.scenarios[1];
    assert!(parent.events[0].enabled());
    assert!(!child.events[0].enabled());
    assert_eq!(child.events[0].name(), "Deposit");
    assert_eq!(resolved_event_occurrences(&config, parent).len(), 1);
    assert!(resolved_event_occurrences(&config, child).is_empty());

    let projection = crate::projection::PlanProjection::from(&config);
    assert_eq!(
        projection.scenarios[0].assets[0].monthly_balances[0].native_balance,
        Decimal::new(110, 0)
    );
    assert_eq!(
        projection.scenarios[1].assets[0].monthly_balances[0].native_balance,
        Decimal::new(60, 0)
    );
    assert!(projection.scenarios[1].asset_events.is_empty());
}

#[test]
fn overrides_event_fields_by_id_and_inherits_omitted_fields() {
    let mut config = event_override_config(
        r#"
[[scenarios.events]]
id = "deposit"
date = "2026-02"
amount = "75"
"#,
    );
    config.validate().unwrap();
    let child = &config.scenarios[1];
    assert_eq!(child.events.len(), 1);
    assert_eq!(child.events[0].name(), "Deposit");
    assert_eq!(child.events[0].asset_id(), "cash");
    assert!(child.events[0].enabled());
    assert_eq!(child.events[0].date(), "2026-02".parse().unwrap());
    assert!(matches!(
        child.events[0],
        Event::AssetAdjustment { amount, .. } if amount == Decimal::new(75, 0)
    ));
    let projection = crate::projection::PlanProjection::from(&config);
    let child = &projection.scenarios[1];
    assert_eq!(child.asset_events.len(), 1);
    assert_eq!(
        child.assets[0].monthly_balances[0].native_balance,
        Decimal::new(60, 0)
    );
    assert_eq!(
        child.assets[0].monthly_balances[1].native_balance,
        Decimal::new(195, 0)
    );
}

#[test]
fn rejects_duplicate_event_overrides_in_one_scenario() {
    let mut config = event_override_config(
        r#"
[[scenarios.events]]
id = "deposit"
enabled = false

[[scenarios.events]]
id = "deposit"
amount = "75"
"#,
    );
    assert!(matches!(
        config.validate(),
        Err(ConfigError::DuplicateEventId { scenario_id, event_id })
            if scenario_id == "child" && event_id == "deposit"
    ));
}

#[test]
fn rejects_incomplete_events_without_an_inherited_match() {
    for id in ["unknown", "   "] {
        let mut config = event_override_config(&format!(
            "[[scenarios.events]]\nid = {id:?}\nenabled = false\n"
        ));
        assert!(config.validate().is_err());
    }
    let mut root =
        recurring_adjustment_config("[[scenarios.events]]\nid = \"unknown\"\nenabled = false\n");
    assert!(matches!(
        root.validate(),
        Err(ConfigError::InvalidEventDefinition { .. })
    ));
}

#[test]
fn inherits_disabled_events_and_reenables_overridden_recurring_events() {
    let mut config: Config =
        toml::from_str(include_str!("../../tests/fixtures/event-overrides.toml")).unwrap();
    config.validate().unwrap();
    // Repeated validation must not resurrect events or duplicate occurrences.
    config.validate().unwrap();
    let scenario = |id: &str| {
        config
            .scenarios
            .iter()
            .find(|scenario| scenario.id == id)
            .unwrap()
    };
    assert_eq!(
        resolved_event_occurrences(&config, scenario("base")).len(),
        12
    );
    for id in ["disabled", "descendant"] {
        assert_eq!(scenario(id).events.len(), 6);
        assert!(scenario(id).events.iter().all(|event| !event.enabled()));
        assert!(resolved_event_occurrences(&config, scenario(id)).is_empty());
    }
    let revived = scenario("revived");
    assert_eq!(
        revived
            .events
            .iter()
            .filter(|event| event.enabled())
            .count(),
        1
    );
    let occurrences = resolved_event_occurrences(&config, revived);
    assert_eq!(occurrences.len(), 3);
    for (occurrence, date) in occurrences.iter().zip(["2026-01", "2026-02", "2026-03"]) {
        assert_eq!(occurrence.date, date.parse().unwrap());
        assert!(matches!(
            occurrence.event,
            Event::AssetAdjustment { amount, .. } if *amount == Decimal::new(75, 0)
        ));
    }

    let projection = crate::projection::PlanProjection::from(&config);
    for id in ["disabled", "descendant", "revived"] {
        let scenario = projection
            .scenarios
            .iter()
            .find(|scenario| scenario.id() == id)
            .unwrap();
        let monthly_increment = if id == "revived" { 135 } else { 60 };
        for (index, month) in scenario.assets[0].monthly_balances.iter().enumerate() {
            assert_eq!(
                month.native_balance,
                Decimal::from(monthly_increment * (index + 1))
            );
            assert_eq!(month.monthly_contribution.amount, Decimal::new(100, 0));
            assert_eq!(month.monthly_withdrawal.amount, Decimal::new(40, 0));
        }
        assert_eq!(
            scenario.asset_events.len(),
            if id == "revived" { 3 } else { 0 }
        );
    }
}

#[test]
fn preserves_override_positions_and_replaces_nested_event_objects() {
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "setter"
name = "Set contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "200", currency = "USD" }

[[scenarios.events]]
id = "adjustment"
name = "Adjust contribution"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { rate = "0.1" }
recurrence = { every = 1, unit = "months", until = "2026-02" }

[[scenarios]]
id = "child"
name = "Child"
extends = "base"

# A new event appends even when it is declared before the overrides.
[[scenarios.events]]
id = "new-adjustment"
name = "New adjustment"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "5", currency = "USD" }

# Overrides are intentionally declared in reverse inherited order.
[[scenarios.events]]
id = "adjustment"
monthly_contribution = { amount = "10", currency = "USD" }
recurrence = { every = 2, unit = "months" }

[[scenarios.events]]
id = "setter"
monthly_contribution = { amount = "100", currency = "USD" }
"#,
    );
    config.validate().unwrap();
    let child = &config.scenarios[1];
    assert_eq!(
        child.events.iter().map(Event::id).collect::<Vec<_>>(),
        ["setter", "adjustment", "new-adjustment"]
    );
    let occurrences = resolved_event_occurrences(&config, child);
    assert_eq!(occurrences.len(), 4);
    assert_eq!(occurrences[3].date, "2026-03".parse().unwrap());
    assert_eq!(child.events[1].recurrence().unwrap().until, None);
    let projection = crate::projection::PlanProjection::from(&config);
    for (month, amount) in projection.scenarios[1].assets[0]
        .monthly_balances
        .iter()
        .zip([115, 115, 125])
    {
        assert_eq!(month.monthly_contribution.amount, Decimal::from(amount));
    }
    // Child object replacements must not modify the parent's objects.
    for (month, amount) in projection.scenarios[0].assets[0]
        .monthly_balances
        .iter()
        .zip([220, 242, 242])
    {
        assert_eq!(month.monthly_contribution.amount, Decimal::from(amount));
    }
}

#[test]
fn validates_disabled_event_fields_and_enabled_boolean() {
    for fields in [
        "enabled = \"false\"",
        "enabled = false\namount = \"not-a-number\"",
        "enabled = false\ndate = \"2025-12\"",
        "enabled = false\nasset_id = \"unknown\"",
    ] {
        let mut config = event_override_config(&format!(
            "[[scenarios.events]]\nid = \"deposit\"\n{fields}\n"
        ));
        assert!(
            config.validate().is_err(),
            "accepted invalid fields: {fields}"
        );
    }
    let mut config = recurring_adjustment_config(
        r#"
[[scenarios.events]]
id = "disabled-new-event"
name = "Disabled new event"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "50"
enabled = false
"#,
    );
    config.validate().unwrap();
    assert!(resolved_event_occurrences(&config, &config.scenarios[0]).is_empty());
}
