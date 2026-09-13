use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, de};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: u32,
    pub plan: Plan,
    #[serde(default)]
    pub conversion_rates: Vec<ConversionRate>,
    #[serde(default)]
    pub milestones: Vec<TotalBalanceMilestone>,
    #[serde(default)]
    pub future_living_costs: Vec<FutureLivingCost>,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub currency: Currency,
    pub start: Month,
    pub end: Month,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Currency(String);

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Deserialize)]
pub struct ConversionRate {
    pub from: Currency,
    pub to: Currency,
    #[serde(deserialize_with = "deserialize_rate")]
    pub rate: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    // Stored for later projection calculations.
    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_decimal")]
    pub annual_inflation: Decimal,
    pub extends: Option<String>,
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub milestones: Vec<AssetMilestone>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    // Stored for later conversion; asset-currency connectivity is not validated yet.
    #[allow(dead_code)]
    pub currency: Currency,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub initial_value: Decimal,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub monthly_expected_return: Decimal,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub monthly_contribution: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    AssetAdjustment {
        id: String,
        date: Month,
        asset_id: String,
        // Stored for later event application; signed values are valid.
        #[allow(dead_code)]
        #[serde(deserialize_with = "deserialize_decimal")]
        amount: Decimal,
    },
    SetMonthlyContribution {
        id: String,
        date: Month,
        asset_id: String,
        #[serde(deserialize_with = "deserialize_decimal")]
        amount: Decimal,
    },
    SetMonthlyExpectedReturn {
        id: String,
        date: Month,
        asset_id: String,
        #[serde(deserialize_with = "deserialize_decimal")]
        rate: Decimal,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TotalBalanceMilestone {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub target: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetMilestone {
    pub id: String,
    pub name: String,
    pub asset_id: String,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub target: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FutureLivingCost {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub monthly_cost: Decimal,
}

impl Event {
    fn id(&self) -> &str {
        match self {
            Self::AssetAdjustment { id, .. }
            | Self::SetMonthlyContribution { id, .. }
            | Self::SetMonthlyExpectedReturn { id, .. } => id,
        }
    }

    fn date(&self) -> Month {
        match self {
            Self::AssetAdjustment { date, .. }
            | Self::SetMonthlyContribution { date, .. }
            | Self::SetMonthlyExpectedReturn { date, .. } => *date,
        }
    }

    fn negative_forward_value(&self) -> Option<&'static str> {
        match self {
            Self::SetMonthlyContribution { amount, .. } if *amount < Decimal::ZERO => {
                Some("amount")
            }
            Self::SetMonthlyExpectedReturn { rate, .. } if *rate < Decimal::ZERO => Some("rate"),
            _ => None,
        }
    }
}

fn deserialize_rate<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_decimal(deserializer)
        .map_err(|_| de::Error::custom("rate must be a decimal string"))
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Decimal::from_str(&value).map_err(|_| de::Error::custom("value must be a decimal string"))
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Month {
    year: u16,
    month: u8,
}

impl fmt::Display for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04}-{:02}", self.year, self.month)
    }
}

impl FromStr for Month {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != 7
            || bytes[4] != b'-'
            || !bytes[..4].iter().all(u8::is_ascii_digit)
            || !bytes[5..].iter().all(u8::is_ascii_digit)
        {
            return Err("month must use the `YYYY-MM` format");
        }

        let year = value[..4]
            .parse::<u16>()
            .expect("four ASCII digits always parse as u16");
        let month = value[5..]
            .parse::<u8>()
            .expect("two ASCII digits always parse as u8");

        if year == 0 || !(1..=12).contains(&month) {
            return Err("month must use the `YYYY-MM` format with a month from `01` to `12`");
        }

        Ok(Self { year, month })
    }
}

impl<'de> Deserialize<'de> for Month {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    UnsupportedVersion {
        version: u32,
    },
    InvalidPlanRange {
        start: Month,
        end: Month,
    },
    InvalidConversionRate {
        from: Currency,
        to: Currency,
    },
    InvalidConversionRateCurrencies {
        from: Currency,
        to: Currency,
        plan_currency: Currency,
    },
    DuplicateConversionRate {
        from: Currency,
        to: Currency,
    },
    NoScenarios,
    BlankScenarioId,
    BlankScenarioName {
        id: String,
    },
    DuplicateScenarioId {
        id: String,
    },
    UnknownScenarioParent {
        id: String,
        parent_id: String,
    },
    ScenarioSelfExtension {
        id: String,
    },
    ScenarioExtensionCycle {
        id: String,
    },
    NoScenarioAssets {
        scenario_id: String,
    },
    BlankAssetId {
        scenario_id: String,
    },
    BlankAssetName {
        scenario_id: String,
        asset_id: String,
    },
    DuplicateAssetId {
        scenario_id: String,
        asset_id: String,
    },
    NegativeAssetValue {
        scenario_id: String,
        asset_id: String,
        field: &'static str,
    },
    BlankEventId {
        scenario_id: String,
    },
    DuplicateEventId {
        scenario_id: String,
        event_id: String,
    },
    EventOutsidePlan {
        scenario_id: String,
        event_id: String,
        date: Month,
        start: Month,
        end: Month,
    },
    UnknownEventAsset {
        scenario_id: String,
        event_id: String,
        asset_id: String,
    },
    NegativeEventValue {
        scenario_id: String,
        event_id: String,
        field: &'static str,
    },
    BlankTotalBalanceMilestoneId,
    BlankTotalBalanceMilestoneName {
        id: String,
    },
    DuplicateTotalBalanceMilestoneId {
        id: String,
    },
    NonpositiveTotalBalanceMilestoneTarget {
        id: String,
    },
    BlankFutureLivingCostId,
    BlankFutureLivingCostName {
        id: String,
    },
    DuplicateFutureLivingCostId {
        id: String,
    },
    NonpositiveFutureLivingCost {
        id: String,
    },
    BlankMilestoneId {
        scenario_id: String,
    },
    BlankMilestoneName {
        scenario_id: String,
        milestone_id: String,
    },
    DuplicateMilestoneId {
        scenario_id: String,
        milestone_id: String,
    },
    NonpositiveMilestoneTarget {
        scenario_id: String,
        milestone_id: String,
    },
    UnknownMilestoneAsset {
        scenario_id: String,
        milestone_id: String,
        asset_id: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read configuration file `{}`: {source}",
                    path.display()
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "failed to parse configuration file `{}`: {}",
                    path.display(),
                    source.message()
                )
            }
            Self::UnsupportedVersion { version } => {
                write!(
                    formatter,
                    "unsupported configuration version {version}; supported version: 1"
                )
            }
            Self::InvalidPlanRange { start, end } => {
                write!(
                    formatter,
                    "plan end month {end} is before start month {start}"
                )
            }
            Self::InvalidConversionRate { from, to } => {
                write!(
                    formatter,
                    "conversion rate `{from}` → `{to}` must be greater than zero"
                )
            }
            Self::InvalidConversionRateCurrencies {
                from,
                to,
                plan_currency,
            } => {
                write!(
                    formatter,
                    "conversion rate `{from}` → `{to}` must have exactly one endpoint matching plan currency `{plan_currency}`"
                )
            }
            Self::DuplicateConversionRate { from, to } => {
                write!(formatter, "duplicate conversion rate `{from}` → `{to}`")
            }
            Self::NoScenarios => {
                write!(formatter, "configuration must define at least one scenario")
            }
            Self::BlankScenarioId => write!(formatter, "scenario id must not be blank"),
            Self::BlankScenarioName { id } => {
                write!(formatter, "scenario `{id}` name must not be blank")
            }
            Self::DuplicateScenarioId { id } => {
                write!(formatter, "duplicate scenario id `{id}`")
            }
            Self::UnknownScenarioParent { id, parent_id } => {
                write!(
                    formatter,
                    "scenario `{id}` extends unknown scenario `{parent_id}`"
                )
            }
            Self::ScenarioSelfExtension { id } => {
                write!(formatter, "scenario `{id}` cannot extend itself")
            }
            Self::ScenarioExtensionCycle { id } => {
                write!(formatter, "scenario extension cycle includes `{id}`")
            }
            Self::NoScenarioAssets { scenario_id } => {
                write!(
                    formatter,
                    "scenario `{scenario_id}` must define at least one asset"
                )
            }
            Self::BlankAssetId { scenario_id } => {
                write!(
                    formatter,
                    "scenario `{scenario_id}` has an asset with a blank id"
                )
            }
            Self::BlankAssetName {
                scenario_id,
                asset_id,
            } => write!(
                formatter,
                "asset `{asset_id}` in scenario `{scenario_id}` has a blank name"
            ),
            Self::DuplicateAssetId {
                scenario_id,
                asset_id,
            } => write!(
                formatter,
                "duplicate asset id `{asset_id}` in scenario `{scenario_id}`"
            ),
            Self::NegativeAssetValue {
                scenario_id,
                asset_id,
                field,
            } => write!(
                formatter,
                "asset `{asset_id}` in scenario `{scenario_id}` has a negative `{field}`"
            ),
            Self::BlankEventId { scenario_id } => {
                write!(
                    formatter,
                    "scenario `{scenario_id}` has an event with a blank id"
                )
            }
            Self::DuplicateEventId {
                scenario_id,
                event_id,
            } => write!(
                formatter,
                "duplicate event id `{event_id}` in scenario `{scenario_id}`"
            ),
            Self::EventOutsidePlan {
                scenario_id,
                event_id,
                date,
                start,
                end,
            } => write!(
                formatter,
                "event `{event_id}` in scenario `{scenario_id}` is dated {date}, outside the plan range {start} through {end}"
            ),
            Self::UnknownEventAsset {
                scenario_id,
                event_id,
                asset_id,
            } => write!(
                formatter,
                "event `{event_id}` in scenario `{scenario_id}` targets unknown asset `{asset_id}`"
            ),
            Self::NegativeEventValue {
                scenario_id,
                event_id,
                field,
            } => write!(
                formatter,
                "event `{event_id}` in scenario `{scenario_id}` has a negative `{field}`"
            ),
            Self::BlankTotalBalanceMilestoneId => {
                write!(
                    formatter,
                    "configuration has a total-balance milestone with a blank id"
                )
            }
            Self::BlankTotalBalanceMilestoneName { id } => {
                write!(formatter, "total-balance milestone `{id}` has a blank name")
            }
            Self::DuplicateTotalBalanceMilestoneId { id } => {
                write!(formatter, "duplicate total-balance milestone id `{id}`")
            }
            Self::NonpositiveTotalBalanceMilestoneTarget { id } => {
                write!(
                    formatter,
                    "total-balance milestone `{id}` must have a positive target"
                )
            }
            Self::BlankFutureLivingCostId => {
                write!(
                    formatter,
                    "configuration has a future living cost with a blank id"
                )
            }
            Self::BlankFutureLivingCostName { id } => {
                write!(formatter, "future living cost `{id}` has a blank name")
            }
            Self::DuplicateFutureLivingCostId { id } => {
                write!(formatter, "duplicate future living cost id `{id}`")
            }
            Self::NonpositiveFutureLivingCost { id } => {
                write!(
                    formatter,
                    "future living cost `{id}` must have a positive monthly cost"
                )
            }
            Self::BlankMilestoneId { scenario_id } => write!(
                formatter,
                "scenario `{scenario_id}` has a milestone with a blank id"
            ),
            Self::BlankMilestoneName {
                scenario_id,
                milestone_id,
            } => write!(
                formatter,
                "milestone `{milestone_id}` in scenario `{scenario_id}` has a blank name"
            ),
            Self::DuplicateMilestoneId {
                scenario_id,
                milestone_id,
            } => write!(
                formatter,
                "duplicate milestone id `{milestone_id}` in scenario `{scenario_id}`"
            ),
            Self::NonpositiveMilestoneTarget {
                scenario_id,
                milestone_id,
            } => write!(
                formatter,
                "milestone `{milestone_id}` in scenario `{scenario_id}` must have a positive target"
            ),
            Self::UnknownMilestoneAsset {
                scenario_id,
                milestone_id,
                asset_id,
            } => write!(
                formatter,
                "milestone `{milestone_id}` in scenario `{scenario_id}` targets unknown asset `{asset_id}`"
            ),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::UnsupportedVersion { .. }
            | Self::InvalidPlanRange { .. }
            | Self::InvalidConversionRate { .. }
            | Self::InvalidConversionRateCurrencies { .. }
            | Self::DuplicateConversionRate { .. }
            | Self::NoScenarios
            | Self::BlankScenarioId
            | Self::BlankScenarioName { .. }
            | Self::DuplicateScenarioId { .. }
            | Self::UnknownScenarioParent { .. }
            | Self::ScenarioSelfExtension { .. }
            | Self::ScenarioExtensionCycle { .. }
            | Self::NoScenarioAssets { .. }
            | Self::BlankAssetId { .. }
            | Self::BlankAssetName { .. }
            | Self::DuplicateAssetId { .. }
            | Self::NegativeAssetValue { .. }
            | Self::BlankEventId { .. }
            | Self::DuplicateEventId { .. }
            | Self::EventOutsidePlan { .. }
            | Self::UnknownEventAsset { .. }
            | Self::NegativeEventValue { .. }
            | Self::BlankTotalBalanceMilestoneId
            | Self::BlankTotalBalanceMilestoneName { .. }
            | Self::DuplicateTotalBalanceMilestoneId { .. }
            | Self::NonpositiveTotalBalanceMilestoneTarget { .. }
            | Self::BlankFutureLivingCostId
            | Self::BlankFutureLivingCostName { .. }
            | Self::DuplicateFutureLivingCostId { .. }
            | Self::NonpositiveFutureLivingCost { .. }
            | Self::BlankMilestoneId { .. }
            | Self::BlankMilestoneName { .. }
            | Self::DuplicateMilestoneId { .. }
            | Self::NonpositiveMilestoneTarget { .. }
            | Self::UnknownMilestoneAsset { .. } => None,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let config: Self = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(ConfigError::UnsupportedVersion {
                version: self.version,
            });
        }

        self.plan.validate()?;
        self.validate_conversion_rates()?;
        self.validate_total_balance_milestones()?;
        self.validate_future_living_costs()?;
        self.validate_scenarios()
    }

    fn validate_conversion_rates(&self) -> Result<(), ConfigError> {
        let mut seen = HashSet::new();

        for conversion_rate in &self.conversion_rates {
            if conversion_rate.rate <= Decimal::ZERO {
                return Err(ConfigError::InvalidConversionRate {
                    from: conversion_rate.from.clone(),
                    to: conversion_rate.to.clone(),
                });
            }

            let has_plan_currency_endpoint = conversion_rate.from == self.plan.currency
                || conversion_rate.to == self.plan.currency;
            let has_single_plan_currency_endpoint =
                has_plan_currency_endpoint && conversion_rate.from != conversion_rate.to;
            if !has_single_plan_currency_endpoint {
                return Err(ConfigError::InvalidConversionRateCurrencies {
                    from: conversion_rate.from.clone(),
                    to: conversion_rate.to.clone(),
                    plan_currency: self.plan.currency.clone(),
                });
            }

            if !seen.insert((&conversion_rate.from, &conversion_rate.to)) {
                return Err(ConfigError::DuplicateConversionRate {
                    from: conversion_rate.from.clone(),
                    to: conversion_rate.to.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_total_balance_milestones(&self) -> Result<(), ConfigError> {
        let mut milestone_ids = HashSet::new();

        for milestone in &self.milestones {
            if milestone.id.trim().is_empty() {
                return Err(ConfigError::BlankTotalBalanceMilestoneId);
            }
            if milestone.name.trim().is_empty() {
                return Err(ConfigError::BlankTotalBalanceMilestoneName {
                    id: milestone.id.clone(),
                });
            }
            if !milestone_ids.insert(milestone.id.as_str()) {
                return Err(ConfigError::DuplicateTotalBalanceMilestoneId {
                    id: milestone.id.clone(),
                });
            }
            if milestone.target <= Decimal::ZERO {
                return Err(ConfigError::NonpositiveTotalBalanceMilestoneTarget {
                    id: milestone.id.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_future_living_costs(&self) -> Result<(), ConfigError> {
        let mut cost_ids = HashSet::new();

        for cost in &self.future_living_costs {
            if cost.id.trim().is_empty() {
                return Err(ConfigError::BlankFutureLivingCostId);
            }
            if cost.name.trim().is_empty() {
                return Err(ConfigError::BlankFutureLivingCostName {
                    id: cost.id.clone(),
                });
            }
            if !cost_ids.insert(cost.id.as_str()) {
                return Err(ConfigError::DuplicateFutureLivingCostId {
                    id: cost.id.clone(),
                });
            }
            if cost.monthly_cost <= Decimal::ZERO {
                return Err(ConfigError::NonpositiveFutureLivingCost {
                    id: cost.id.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_scenarios(&self) -> Result<(), ConfigError> {
        if self.scenarios.is_empty() {
            return Err(ConfigError::NoScenarios);
        }

        let mut scenario_by_id = HashMap::new();
        for scenario in &self.scenarios {
            if scenario.id.trim().is_empty() {
                return Err(ConfigError::BlankScenarioId);
            }
            if scenario.name.trim().is_empty() {
                return Err(ConfigError::BlankScenarioName {
                    id: scenario.id.clone(),
                });
            }
            if scenario_by_id
                .insert(scenario.id.as_str(), scenario)
                .is_some()
            {
                return Err(ConfigError::DuplicateScenarioId {
                    id: scenario.id.clone(),
                });
            }
        }

        for scenario in &self.scenarios {
            if scenario.extends.as_deref() == Some(scenario.id.as_str()) {
                return Err(ConfigError::ScenarioSelfExtension {
                    id: scenario.id.clone(),
                });
            }

            let mut visited = HashSet::from([scenario.id.as_str()]);
            let mut current = scenario;
            while let Some(parent_id) = current.extends.as_deref() {
                let Some(parent) = scenario_by_id.get(parent_id) else {
                    return Err(ConfigError::UnknownScenarioParent {
                        id: current.id.clone(),
                        parent_id: parent_id.to_owned(),
                    });
                };

                if !visited.insert(parent_id) {
                    return Err(ConfigError::ScenarioExtensionCycle {
                        id: parent_id.to_owned(),
                    });
                }
                current = parent;
            }
        }

        for scenario in &self.scenarios {
            self.validate_assets(scenario)?;
        }
        for scenario in &self.scenarios {
            self.validate_events(scenario, &scenario_by_id)?;
        }
        for scenario in &self.scenarios {
            self.validate_milestones(scenario, &scenario_by_id)?;
        }

        Ok(())
    }

    fn validate_assets(&self, scenario: &Scenario) -> Result<(), ConfigError> {
        if scenario.assets.is_empty() {
            return Err(ConfigError::NoScenarioAssets {
                scenario_id: scenario.id.clone(),
            });
        }

        let mut asset_ids = HashSet::new();
        for asset in &scenario.assets {
            if asset.id.trim().is_empty() {
                return Err(ConfigError::BlankAssetId {
                    scenario_id: scenario.id.clone(),
                });
            }
            if asset.name.trim().is_empty() {
                return Err(ConfigError::BlankAssetName {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                });
            }
            if !asset_ids.insert(asset.id.as_str()) {
                return Err(ConfigError::DuplicateAssetId {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                });
            }

            for (field, value) in [
                ("initial_value", asset.initial_value),
                ("monthly_expected_return", asset.monthly_expected_return),
                ("monthly_contribution", asset.monthly_contribution),
            ] {
                if value < Decimal::ZERO {
                    return Err(ConfigError::NegativeAssetValue {
                        scenario_id: scenario.id.clone(),
                        asset_id: asset.id.clone(),
                        field,
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_events(
        &self,
        scenario: &Scenario,
        scenario_by_id: &HashMap<&str, &Scenario>,
    ) -> Result<(), ConfigError> {
        let mut event_ids = HashSet::new();

        for event in &scenario.events {
            if event.id().trim().is_empty() {
                return Err(ConfigError::BlankEventId {
                    scenario_id: scenario.id.clone(),
                });
            }
            if !event_ids.insert(event.id()) {
                return Err(ConfigError::DuplicateEventId {
                    scenario_id: scenario.id.clone(),
                    event_id: event.id().to_owned(),
                });
            }

            let date = event.date();
            if date < self.plan.start || date > self.plan.end {
                return Err(ConfigError::EventOutsidePlan {
                    scenario_id: scenario.id.clone(),
                    event_id: event.id().to_owned(),
                    date,
                    start: self.plan.start,
                    end: self.plan.end,
                });
            }
            let asset_id = match event {
                Event::AssetAdjustment { asset_id, .. }
                | Event::SetMonthlyContribution { asset_id, .. }
                | Event::SetMonthlyExpectedReturn { asset_id, .. } => asset_id,
            };
            if !self.scenario_has_asset(scenario, asset_id, scenario_by_id) {
                return Err(ConfigError::UnknownEventAsset {
                    scenario_id: scenario.id.clone(),
                    event_id: event.id().to_owned(),
                    asset_id: asset_id.to_owned(),
                });
            }
            if let Some(field) = event.negative_forward_value() {
                return Err(ConfigError::NegativeEventValue {
                    scenario_id: scenario.id.clone(),
                    event_id: event.id().to_owned(),
                    field,
                });
            }
        }

        Ok(())
    }

    fn validate_milestones(
        &self,
        scenario: &Scenario,
        scenario_by_id: &HashMap<&str, &Scenario>,
    ) -> Result<(), ConfigError> {
        let mut milestone_ids = HashSet::new();

        for milestone in &scenario.milestones {
            if milestone.id.trim().is_empty() {
                return Err(ConfigError::BlankMilestoneId {
                    scenario_id: scenario.id.clone(),
                });
            }
            if milestone.name.trim().is_empty() {
                return Err(ConfigError::BlankMilestoneName {
                    scenario_id: scenario.id.clone(),
                    milestone_id: milestone.id.clone(),
                });
            }
            if !milestone_ids.insert(milestone.id.as_str()) {
                return Err(ConfigError::DuplicateMilestoneId {
                    scenario_id: scenario.id.clone(),
                    milestone_id: milestone.id.clone(),
                });
            }
            if milestone.target <= Decimal::ZERO {
                return Err(ConfigError::NonpositiveMilestoneTarget {
                    scenario_id: scenario.id.clone(),
                    milestone_id: milestone.id.clone(),
                });
            }
            if !self.scenario_has_asset(scenario, &milestone.asset_id, scenario_by_id) {
                return Err(ConfigError::UnknownMilestoneAsset {
                    scenario_id: scenario.id.clone(),
                    milestone_id: milestone.id.clone(),
                    asset_id: milestone.asset_id.clone(),
                });
            }
        }

        Ok(())
    }

    fn scenario_has_asset(
        &self,
        scenario: &Scenario,
        asset_id: &str,
        scenario_by_id: &HashMap<&str, &Scenario>,
    ) -> bool {
        let mut current = scenario;

        loop {
            if current.assets.iter().any(|asset| asset.id == asset_id) {
                return true;
            }

            let Some(parent_id) = current.extends.as_deref() else {
                return false;
            };
            let Some(parent) = scenario_by_id.get(parent_id) else {
                return false;
            };
            current = parent;
        }
    }
}

impl Plan {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.end < self.start {
            return Err(ConfigError::InvalidPlanRange {
                start: self.start,
                end: self.end,
            });
        }

        Ok(())
    }
}
