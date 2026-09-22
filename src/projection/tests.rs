use rust_decimal::Decimal;

use super::*;

fn config(source: &str) -> Config {
    let mut config: Config = toml::from_str(source).expect("test configuration parses");
    config.validate().expect("test configuration validates");
    config
}

#[test]
fn projects_inclusive_monthly_ending_balances() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances.len(), 2);
    assert_eq!(balances[0].month.to_string(), "2026-01");
    assert_eq!(balances[0].native_balance, Decimal::new(110, 0));
    assert_eq!(balances[0].plan_balance, Decimal::new(110, 0));
    assert_eq!(balances[0].native_passive_income, Decimal::ZERO);
    assert_eq!(balances[0].plan_passive_income, Decimal::ZERO);
    assert_eq!(balances[1].month.to_string(), "2026-02");
    assert_eq!(balances[1].native_balance, Decimal::new(120, 0));
    assert_eq!(balances[1].native_passive_income, Decimal::ZERO);
    assert_eq!(balances[1].plan_passive_income, Decimal::ZERO);
    assert_eq!(
        projection.scenarios[0].total_net_worth[1].balance,
        Decimal::new(120, 0)
    );
}

#[test]
fn compounds_annual_expected_returns_over_twelve_months() {
    let config = config(
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
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0.1"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(
        balances
            .last()
            .expect("twelve monthly balances")
            .native_balance
            .round_dp(20),
        Decimal::new(110, 0)
    );
    assert!(
        balances
            .iter()
            .all(|balance| balance.annual_expected_return == Decimal::new(1, 1))
    );
}

#[test]
fn inflates_end_of_plan_living_costs_across_inclusive_months() {
    let source = r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[[future_living_costs]]
id = "living"
name = "Living"
monthly_cost = "100"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0.1"

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
"#;

    let one_month_config = config(source);
    let one_month_projection = PlanProjection::from(&one_month_config);
    assert!(
        one_month_projection.scenarios[0]
            .future_living_costs
            .nominal_monthly_total
            .plan_amount()
            > Decimal::new(100, 0)
    );

    let twelve_month_config = config(&source.replace("end = \"2026-01\"", "end = \"2026-12\""));
    let twelve_month_projection = PlanProjection::from(&twelve_month_config);
    assert_eq!(
        twelve_month_projection.scenarios[0]
            .future_living_costs
            .nominal_monthly_total
            .plan_amount(),
        Decimal::new(110, 0)
    );

    let overridden_config = config(
        &source
            .replace("end = \"2026-01\"", "end = \"2026-12\"")
            .replace(
                "monthly_cost = \"100\"",
                "annual_inflation = \"0.2\"\nmonthly_cost = \"100\"",
            ),
    );
    let overridden_projection = PlanProjection::from(&overridden_config);
    assert_eq!(
        overridden_projection.scenarios[0]
            .future_living_costs
            .nominal_monthly_total
            .plan_amount(),
        Decimal::new(120, 0)
    );
}

#[test]
fn projects_converted_assets_in_plan_currency() {
    let config = config(
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
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "exchange-balance"
name = "Exchange balance"
currency = "USD"
value = "2500"

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

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "deposit"
name = "Deposit"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "1"
"#;

    assert!(toml::from_str::<Config>(&format!("version = 1\n\n{source}")).is_err());
    assert!(toml::from_str::<Config>(&source.replace("name = \"Deposit\"\n", "")).is_err());

    let mut blank_name: Config = toml::from_str(&source.replace("Deposit", "   "))
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
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

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
        Decimal::new(115, 0)
    );
}

#[test]
fn applies_adjustments_after_contributions_and_return_settings() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

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
monthly_contribution = { amount = "100", currency = "USD" }

[[scenarios.events]]
id = "ignored-return-change"
name = "Ignored return change"
type = "set_annual_expected_return"
date = "2026-03"
asset_id = "cash"
rate = "0.5"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances[0].native_balance, Decimal::new(115, 0));
    assert_eq!(balances[1].native_balance, Decimal::new(105, 0));
    assert_eq!(balances[2].annual_expected_return, Decimal::new(5, 1));
    assert_eq!(
        balances[2].native_balance,
        balances[1].native_balance * (Decimal::ONE + monthly_rate(Decimal::new(5, 1)))
            + Decimal::new(105, 0)
    );
}

#[test]
fn contribution_settings_take_effect_immediately_persist_and_last_setting_wins() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

[[scenarios.events]]
id = "first-january-setting"
name = "First January setting"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "20", currency = "USD" }

[[scenarios.events]]
id = "second-january-setting"
name = "Second January setting"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "30", currency = "USD" }

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
            .all(|balance| balance.monthly_contribution.amount == Decimal::new(30, 0))
    );
}

#[test]
fn inherits_adjustments_for_a_child_replacement_asset() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "200"

[[scenarios.events]]
id = "child-return"
name = "Child return"
type = "set_annual_expected_return"
date = "2026-01"
asset_id = "brokerage"
rate = "0.2"

[[scenarios.events]]
id = "child-contribution"
name = "Child contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "brokerage"
monthly_contribution = { amount = "30", currency = "USD" }

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "brokerage"
name = "Parent brokerage"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

[[scenarios.events]]
id = "parent-return"
name = "Parent return"
type = "set_annual_expected_return"
date = "2026-01"
asset_id = "brokerage"
rate = "0.1"

[[scenarios.events]]
id = "parent-contribution"
name = "Parent contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "brokerage"
monthly_contribution = { amount = "20", currency = "USD" }

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
        Decimal::new(200, 0) * (Decimal::ONE + monthly_rate(Decimal::new(2, 1)))
            + Decimal::new(80, 0)
    );
    assert_eq!(
        child.assets[0].monthly_balances[0]
            .monthly_contribution
            .amount,
        Decimal::new(30, 0)
    );
    assert_eq!(
        parent.assets[0].monthly_balances[0].native_balance,
        Decimal::new(100, 0) * (Decimal::ONE + monthly_rate(Decimal::new(1, 1)))
            + Decimal::new(70, 0)
    );
}

#[test]
fn resolves_parent_assets_before_child_listed_earlier() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "200"

[[scenarios.assets]]
id = "savings"
name = "Savings"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "50"

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "brokerage"
name = "Parent brokerage"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

[[scenarios.assets]]
id = "bitcoin"
name = "Bitcoin"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "5"
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
fn converts_plan_currency_contributions_and_applies_currency_settings_immediately() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-02"

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
monthly_contribution = { amount = "0.1", currency = "BTC" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "BTC"
value = "0"

[[scenarios.events]]
id = "switch-contribution-currency"
name = "Switch contribution currency"
type = "set_monthly_contribution"
date = "2026-02"
asset_id = "bitcoin"
monthly_contribution = { amount = "100", currency = "USD" }
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;

    assert_eq!(balances[0].native_balance, Decimal::new(1, 1));
    assert_eq!(balances[0].monthly_contribution.currency.to_string(), "BTC");
    assert_eq!(balances[1].native_balance, Decimal::new(11, 2));
    assert_eq!(
        balances[1].monthly_contribution.amount,
        Decimal::new(100, 0)
    );
    assert_eq!(balances[1].monthly_contribution.currency.to_string(), "USD");
}

#[test]
fn accumulates_parent_and_child_adjustments_on_an_inherited_asset() {
    let config = config(
        r#"
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
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

[[scenarios.events]]
id = "parent-adjustment"
name = "Parent adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "10"

[[scenarios.events]]
id = "parent-contribution-adjustment"
name = "Parent contribution adjustment"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "10", currency = "USD" }

[[scenarios]]
id = "middle"
name = "Middle"
annual_inflation = "0"
extends = "parent"

[[scenarios.assets]]
id = "middle-marker"
name = "Middle marker"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "middle-adjustment"
name = "Middle adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "20"

[[scenarios.events]]
id = "middle-contribution-adjustment"
name = "Middle contribution adjustment"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { amount = "20", currency = "USD" }

[[scenarios]]
id = "child"
name = "Child"
annual_inflation = "0"
extends = "middle"

[[scenarios.assets]]
id = "child-marker"
name = "Child marker"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "child-adjustment"
name = "Child adjustment"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "-5"

[[scenarios.events]]
id = "child-contribution-adjustment"
name = "Child contribution adjustment"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "cash"
monthly_contribution = { rate = "-0.5" }
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

    assert_eq!(cash_balance(0), Decimal::new(120, 0));
    assert_eq!(cash_balance(1), Decimal::new(160, 0));
    assert_eq!(cash_balance(2), Decimal::new(140, 0));
}

#[test]
fn projects_assets_only_within_their_inclusive_lifecycle() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-04"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0.03"

[[scenarios.assets]]
id = "new-asset"
name = "New asset"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }
monthly_withdrawal = { amount = "10", currency = "USD" }
starts = "2026-02"
ends = "2026-03"

[[scenarios.assets.holdings]]
id = "holding"
name = "Holding"
currency = "USD"
value = "100"
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;
    assert_eq!(
        balances
            .iter()
            .map(|balance| balance.is_active)
            .collect::<Vec<_>>(),
        [false, true, true, false]
    );
    assert_eq!(balances[0].native_balance, Decimal::ZERO);
    assert_eq!(balances[1].native_balance, Decimal::new(90, 0));
    assert_eq!(balances[2].native_balance, Decimal::new(80, 0));
    assert_eq!(balances[3].native_balance, Decimal::ZERO);
}

#[test]
fn applies_and_persists_absolute_and_relative_recurring_flow_changes() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-04"

[[conversion_rates]]
from = "BTC"
to = "USD"
rate = "100"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "account"
name = "Account"
currency = "BTC"
annual_expected_return = "0"
monthly_contribution = { amount = "100", currency = "USD" }
monthly_withdrawal = { amount = "0.25", currency = "BTC" }

[[scenarios.assets.holdings]]
id = "bitcoin"
name = "Bitcoin"
currency = "BTC"
value = "1"

[[scenarios.events]]
id = "increase-withdrawal"
name = "Increase withdrawal"
date = "2026-02"
type = "set_monthly_withdrawal"
asset_id = "account"
monthly_withdrawal = { amount = "50", currency = "USD" }

[[scenarios.events]]
id = "reduce-withdrawal"
name = "Reduce withdrawal"
date = "2026-02"
type = "adjust_monthly_withdrawal"
asset_id = "account"
monthly_withdrawal = { amount = "-10", currency = "USD" }

[[scenarios.events]]
id = "raise-contribution"
name = "Raise contribution"
date = "2026-02"
type = "adjust_monthly_contribution"
asset_id = "account"
monthly_contribution = { amount = "20", currency = "USD" }

[[scenarios.events]]
id = "percentage-contribution-increase"
name = "Percentage contribution increase"
date = "2026-02"
type = "adjust_monthly_contribution"
asset_id = "account"
monthly_contribution = { rate = "0.1" }

[[scenarios.events]]
id = "percentage-withdrawal-reduction"
name = "Percentage withdrawal reduction"
date = "2026-02"
type = "adjust_monthly_withdrawal"
asset_id = "account"
monthly_withdrawal = { rate = "-0.25" }

[[scenarios.events]]
id = "compound-contribution-increase"
name = "Compound contribution increase"
date = "2026-03"
type = "adjust_monthly_contribution"
asset_id = "account"
monthly_contribution = { rate = "0.1" }

[[scenarios.events]]
id = "stop-withdrawal"
name = "Stop withdrawal"
date = "2026-04"
type = "adjust_monthly_withdrawal"
asset_id = "account"
monthly_withdrawal = { rate = "-1" }

[[scenarios.events]]
id = "increase-zero-withdrawal"
name = "Increase zero withdrawal"
date = "2026-04"
type = "adjust_monthly_withdrawal"
asset_id = "account"
monthly_withdrawal = { rate = "0.5" }
"#,
    );

    let projection = PlanProjection::from(&config);
    let balances = &projection.scenarios[0].assets[0].monthly_balances;
    assert_eq!(balances[0].native_balance, Decimal::new(175, 2));
    assert_eq!(balances[1].native_balance, Decimal::new(277, 2));
    assert_eq!(balances[2].native_balance, Decimal::new(3922, 3));
    assert_eq!(balances[3].native_balance, Decimal::new(5374, 3));
    assert_eq!(balances[0].monthly_withdrawal.currency.to_string(), "BTC");
    assert_eq!(balances[1].monthly_withdrawal.currency.to_string(), "USD");
    assert_eq!(
        balances[2].monthly_contribution.amount,
        Decimal::new(1452, 1)
    );
    assert_eq!(
        balances[3].monthly_contribution.amount,
        Decimal::new(1452, 1)
    );
    assert_eq!(balances[3].monthly_withdrawal.amount, Decimal::ZERO);
}

#[test]
fn repeats_events_through_the_assets_inclusive_end_month() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2028-02"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "account"
name = "Account"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "100", currency = "USD" }
ends = "2028-01"

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "annual-contribution-increase"
name = "Annual contribution increase"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "account"
monthly_contribution = { rate = "0.1" }
recurrence = { every = 1, unit = "years" }

[[scenarios.events]]
id = "periodic-withdrawal-increase"
name = "Periodic withdrawal increase"
type = "adjust_monthly_withdrawal"
date = "2026-03"
asset_id = "account"
monthly_withdrawal = { amount = "10", currency = "USD" }
recurrence = { every = 6, unit = "months" }

[[scenarios.events]]
id = "periodic-purchase"
name = "Periodic purchase"
type = "asset_adjustment"
date = "2026-02"
asset_id = "account"
amount = "-100"
recurrence = { every = 6, unit = "months" }
"#,
    );

    let projection = PlanProjection::from(&config);
    let scenario = &projection.scenarios[0];
    let balances = &scenario.assets[0].monthly_balances;

    assert_eq!(
        balances[0].monthly_contribution.amount,
        Decimal::new(110, 0)
    );
    assert_eq!(
        balances[12].monthly_contribution.amount,
        Decimal::new(121, 0)
    );
    assert_eq!(
        balances[24].monthly_contribution.amount,
        Decimal::new(1331, 1)
    );
    assert!(balances[24].is_active);
    assert!(!balances[25].is_active);
    assert_eq!(balances[2].monthly_withdrawal.amount, Decimal::new(10, 0));
    assert_eq!(balances[8].monthly_withdrawal.amount, Decimal::new(20, 0));
    assert_eq!(balances[14].monthly_withdrawal.amount, Decimal::new(30, 0));
    assert_eq!(balances[20].monthly_withdrawal.amount, Decimal::new(40, 0));
    assert_eq!(scenario.asset_adjustments.len(), 4);
    assert_eq!(scenario.asset_events.len(), 11);
    assert_eq!(scenario.asset_adjustments[0].date.to_string(), "2026-02");
    assert_eq!(scenario.asset_adjustments[3].date.to_string(), "2027-08");
}

#[test]
fn orders_parent_recurring_occurrences_before_child_events_in_the_same_month() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2027-01"

[[scenarios]]
id = "parent"
name = "Parent"
annual_inflation = "0"

[[scenarios.assets]]
id = "account"
name = "Account"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "annual-fixed-increase"
name = "Annual fixed increase"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "account"
monthly_contribution = { amount = "10", currency = "USD" }
recurrence = { every = 1, unit = "years" }

[[scenarios]]
id = "child"
name = "Child"
extends = "parent"

[[scenarios.events]]
id = "halve-contribution"
name = "Halve contribution"
type = "adjust_monthly_contribution"
date = "2027-01"
asset_id = "account"
monthly_contribution = { rate = "-0.5" }
"#,
    );

    let projection = PlanProjection::from(&config);
    let parent = &projection.scenarios[0].assets[0].monthly_balances;
    let child = &projection.scenarios[1].assets[0].monthly_balances;

    assert_eq!(parent[12].monthly_contribution.amount, Decimal::new(20, 0));
    assert_eq!(child[12].monthly_contribution.amount, Decimal::new(10, 0));
}
