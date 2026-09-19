use super::*;

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
    assert_eq!(asset.holdings.len(), 1);
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
