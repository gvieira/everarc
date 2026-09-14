use rust_decimal::Decimal;

use super::*;

fn config(source: &str) -> Config {
    let config: Config = toml::from_str(source).expect("test configuration parses");
    config.validate().expect("test configuration validates");
    config
}

#[test]
fn projects_inclusive_monthly_ending_balances() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-02"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0.1"
monthly_contribution = "10"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances.len(), 2);
    assert_eq!(balances[0].month.to_string(), "2026-01");
    assert_eq!(balances[0].native_balance, Decimal::new(120, 0));
    assert_eq!(balances[0].plan_balance, Decimal::new(120, 0));
    assert_eq!(balances[1].month.to_string(), "2026-02");
    assert_eq!(balances[1].native_balance, Decimal::new(142, 0));
    assert_eq!(
        projection.scenarios[0].total_net_worth[1].balance,
        Decimal::new(142, 0)
    );
}

#[test]
fn projects_converted_assets_in_plan_currency() {
    let config = config(
        r#"
version = 1

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
initial_value = "0.25"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "bitcoin-deposit"
name = "Bitcoin deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "bitcoin"
amount = "0.1"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balance = &projection.scenarios[0].assets[0].monthly_balances[0];

    assert_eq!(balance.native_balance, Decimal::new(35, 2));
    assert_eq!(balance.plan_balance, Decimal::new(3500, 0));
    assert_eq!(
        projection.scenarios[0].total_net_worth[0].balance,
        Decimal::new(3500, 0)
    );
}

#[test]
fn rejects_missing_and_blank_event_names() {
    let source = r#"
version = 1

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
initial_value = "0"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "deposit"
name = "Deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "1"
"#;

    assert!(toml::from_str::<Config>(&source.replace("name = \"Deposit\"\n", "")).is_err());

    let blank_name: Config = toml::from_str(&source.replace("Deposit", "   "))
        .expect("blank names parse before semantic validation");
    assert!(matches!(
        blank_name.validate(),
        Err(crate::config::ConfigError::BlankEventName { .. })
    ));
}

#[test]
fn applies_adjustments_after_returns_and_contributions() {
    let config = config(
        r#"
version = 1

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
initial_value = "100"
monthly_expected_return = "0.1"
monthly_contribution = "10"

[[scenarios.events]]
id = "deposit"
name = "Deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "5"
"#,
    );

    let projection = PlanProjection::from(&config);
    assert_eq!(
        projection.scenarios[0].assets[0].monthly_balances[0].native_balance,
        Decimal::new(125, 0)
    );
}

#[test]
fn applies_adjustments_after_contributions_and_return_settings() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.events]]
id = "january-deposit"
name = "January deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "5"

[[scenarios.events]]
id = "february-withdrawal"
name = "February withdrawal"
type = "asset_adjustment"
date = "2026-02"
asset_id = "cash"
amount = "-20"

[[scenarios.events]]
id = "march-deposit-one"
name = "March deposit one"
type = "asset_adjustment"
date = "2026-03"
asset_id = "cash"
amount = "3"

[[scenarios.events]]
id = "march-deposit-two"
name = "March deposit two"
type = "asset_adjustment"
date = "2026-03"
asset_id = "cash"
amount = "2"

[[scenarios.events]]
id = "ignored-contribution-change"
name = "Ignored contribution change"
type = "set_monthly_contribution"
date = "2026-03"
asset_id = "cash"
amount = "100"

[[scenarios.events]]
id = "ignored-return-change"
name = "Ignored return change"
type = "set_monthly_expected_return"
date = "2026-03"
asset_id = "cash"
rate = "0.5"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances[0].native_balance, Decimal::new(115, 0));
    assert_eq!(balances[1].native_balance, Decimal::new(105, 0));
    assert_eq!(balances[2].native_balance, Decimal::new(2625, 1));
}

#[test]
fn contribution_settings_take_effect_immediately_persist_and_last_setting_wins() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.events]]
id = "first-january-setting"
name = "First January setting"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "cash"
amount = "20"

[[scenarios.events]]
id = "second-january-setting"
name = "Second January setting"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "cash"
amount = "30"

[[scenarios.events]]
id = "march-adjustment"
name = "March adjustment"
type = "asset_adjustment"
date = "2026-03"
asset_id = "cash"
amount = "5"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances[0].native_balance, Decimal::new(130, 0));
    assert_eq!(balances[1].native_balance, Decimal::new(160, 0));
    assert_eq!(balances[2].native_balance, Decimal::new(195, 0));
    assert!(
        balances
            .iter()
            .all(|balance| balance.monthly_contribution == Decimal::new(30, 0))
    );
}

#[test]
fn inherits_adjustments_for_a_child_replacement_asset() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "child"
name = "Child"
annual_inflation = "0"
extends = "parent"

[[scenarios.assets]]
id = "brokerage"
name = "Child brokerage"
currency = "USD"
initial_value = "200"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.events]]
id = "child-return"
name = "Child return"
type = "set_monthly_expected_return"
date = "2026-01"
asset_id = "brokerage"
rate = "0.2"

[[scenarios.events]]
id = "child-contribution"
name = "Child contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "brokerage"
amount = "30"

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "brokerage"
name = "Parent brokerage"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.events]]
id = "parent-return"
name = "Parent return"
type = "set_monthly_expected_return"
date = "2026-01"
asset_id = "brokerage"
rate = "0.1"

[[scenarios.events]]
id = "parent-contribution"
name = "Parent contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "brokerage"
amount = "20"

[[scenarios.events]]
id = "parent-deposit"
name = "Parent deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "brokerage"
amount = "50"
"#,
    );

    let projection = PlanProjection::from(&config);
    let child = &projection.scenarios[0];
    let parent = &projection.scenarios[1];

    assert_eq!(
        child.assets[0].monthly_balances[0].native_balance,
        Decimal::new(320, 0)
    );
    assert_eq!(
        child.assets[0].monthly_balances[0].monthly_contribution,
        Decimal::new(30, 0)
    );
    assert_eq!(
        parent.assets[0].monthly_balances[0].native_balance,
        Decimal::new(180, 0)
    );
}

#[test]
fn resolves_parent_assets_before_child_listed_earlier() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "child"
name = "Child"
annual_inflation = "0"
extends = "parent"

[[scenarios.assets]]
id = "brokerage"
name = "Child brokerage"
currency = "USD"
initial_value = "200"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.assets]]
id = "savings"
name = "Savings"
currency = "USD"
initial_value = "50"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "brokerage"
name = "Parent brokerage"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "10"

[[scenarios.assets]]
id = "bitcoin"
name = "Bitcoin"
currency = "USD"
initial_value = "5"
monthly_expected_return = "0"
monthly_contribution = "0"
"#,
    );

    let projection = PlanProjection::from(&config);
    let child = &projection.scenarios[0];
    let parent = &projection.scenarios[1];

    assert_eq!(
        child
            .assets
            .iter()
            .map(AssetProjection::id)
            .collect::<Vec<_>>(),
        vec!["brokerage", "bitcoin", "savings"]
    );
    assert_eq!(
        child.assets[0].monthly_balances[0].native_balance,
        Decimal::new(210, 0)
    );
    assert_eq!(
        parent.assets[0].monthly_balances[0].native_balance,
        Decimal::new(110, 0)
    );
    assert_eq!(
        child.assets[1].monthly_balances[0].native_balance,
        Decimal::new(5, 0)
    );
    assert_eq!(
        child.assets[2].monthly_balances[0].native_balance,
        Decimal::new(50, 0)
    );
}

#[test]
fn accumulates_parent_and_child_adjustments_on_an_inherited_asset() {
    let config = config(
        r#"
version = 1

[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "parent-adjustment"
name = "Parent adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "10"

[[scenarios]]
id = "middle"
name = "Middle"
annual_inflation = "0"
extends = "parent"

[[scenarios.assets]]
id = "middle-marker"
name = "Middle marker"
currency = "USD"
initial_value = "0"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "middle-adjustment"
name = "Middle adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "20"

[[scenarios]]
id = "child"
name = "Child"
annual_inflation = "0"
extends = "middle"

[[scenarios.assets]]
id = "child-marker"
name = "Child marker"
currency = "USD"
initial_value = "0"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "child-adjustment"
name = "Child adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "-5"
"#,
    );

    let projection = PlanProjection::from(&config);
    let cash_balance = |scenario_index: usize| {
        projection.scenarios[scenario_index]
            .assets
            .iter()
            .find(|asset| asset.id() == "cash")
            .expect("cash is inherited")
            .monthly_balances[0]
            .native_balance
    };

    assert_eq!(cash_balance(0), Decimal::new(110, 0));
    assert_eq!(cash_balance(1), Decimal::new(130, 0));
    assert_eq!(cash_balance(2), Decimal::new(125, 0));
}
