const MAX_SCENARIO_NAME_LENGTH: usize = 32;

use std::{
    collections::{HashMap, HashSet},
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser::SerializeStruct};

const INHERIT_DECIMAL: Decimal = Decimal::MIN;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub plan: Plan,
    #[serde(default)]
    pub conversion_rates: Vec<ConversionRate>,
    #[serde(default)]
    pub milestones: Vec<TotalBalanceMilestone>,
    #[serde(default)]
    pub future_living_costs: Vec<FutureLivingCost>,
    pub display: DisplaySettings,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
pub struct DisplaySettings {
    pub locale: Locale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Locale {
    EnUs,
    PtBr,
}

impl Locale {
    pub fn html_language(self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::PtBr => "pt-BR",
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.html_language())
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "en-US" => Ok(Self::EnUs),
            "pt-BR" => Ok(Self::PtBr),
            _ => Err(de::Error::custom("locale must be `en-US` or `pt-BR`")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub currency: Currency,
    pub start: Month,
    pub end: Month,
}

impl Plan {
    pub fn inclusive_month_count(&self) -> u32 {
        let start = u32::from(self.start.year) * 12 + u32::from(self.start.month - 1);
        let end = u32::from(self.end.year) * 12 + u32::from(self.end.month - 1);
        end - start + 1
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub selected: bool,
    #[serde(default = "inherit_decimal", deserialize_with = "deserialize_decimal")]
    pub annual_inflation: Decimal,
    pub extends: Option<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub milestones: Vec<AssetMilestone>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonthlyContribution {
    #[serde(deserialize_with = "deserialize_decimal")]
    pub amount: Decimal,
    pub currency: Currency,
}

impl Default for MonthlyContribution {
    fn default() -> Self {
        Self {
            amount: INHERIT_DECIMAL,
            currency: Currency::default(),
        }
    }
}

impl Serialize for MonthlyContribution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MonthlyContribution", 2)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.serialize_field("currency", &self.currency)?;
        state.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    #[serde(default)]
    pub name: String,
    // Every asset currency must connect directly to the plan currency.
    #[serde(default)]
    pub currency: Currency,
    #[serde(default = "inherit_decimal", deserialize_with = "deserialize_decimal")]
    pub annual_expected_return: Decimal,
    #[serde(default)]
    pub monthly_contribution: MonthlyContribution,
    #[serde(default)]
    pub holdings: Vec<Holding>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Holding {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub currency: Currency,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub value: Decimal,
}

impl Serialize for Holding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Holding", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("currency", &self.currency)?;
        state.serialize_field("value", &self.value.to_string())?;
        state.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    AssetAdjustment {
        id: String,
        name: String,
        date: Month,
        asset_id: String,
        #[serde(deserialize_with = "deserialize_decimal")]
        amount: Decimal,
    },
    SetMonthlyContribution {
        id: String,
        name: String,
        date: Month,
        asset_id: String,
        #[serde(deserialize_with = "deserialize_decimal")]
        amount: Decimal,
        currency: Currency,
    },
    SetAnnualExpectedReturn {
        id: String,
        name: String,
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

#[derive(Clone, Debug, Deserialize)]
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
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub annual_inflation: Option<Decimal>,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub monthly_cost: Decimal,
}

impl Event {
    fn id(&self) -> &str {
        match self {
            Self::AssetAdjustment { id, .. }
            | Self::SetMonthlyContribution { id, .. }
            | Self::SetAnnualExpectedReturn { id, .. } => id,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::AssetAdjustment { name, .. }
            | Self::SetMonthlyContribution { name, .. }
            | Self::SetAnnualExpectedReturn { name, .. } => name,
        }
    }

    fn date(&self) -> Month {
        match self {
            Self::AssetAdjustment { date, .. }
            | Self::SetMonthlyContribution { date, .. }
            | Self::SetAnnualExpectedReturn { date, .. } => *date,
        }
    }

    fn negative_forward_value(&self) -> Option<&'static str> {
        match self {
            Self::SetMonthlyContribution { amount, .. } if *amount < Decimal::ZERO => {
                Some("amount")
            }
            Self::SetAnnualExpectedReturn { rate, .. } if *rate < Decimal::ZERO => Some("rate"),
            _ => None,
        }
    }
}

fn merge_asset(parent: &Asset, child: &Asset) -> Asset {
    Asset {
        id: child.id.clone(),
        name: if !child.name.is_empty() {
            child.name.clone()
        } else {
            parent.name.clone()
        },
        currency: if !child.currency.0.is_empty() {
            child.currency.clone()
        } else {
            parent.currency.clone()
        },
        annual_expected_return: if child.annual_expected_return != INHERIT_DECIMAL {
            child.annual_expected_return
        } else {
            parent.annual_expected_return
        },
        monthly_contribution: if child.monthly_contribution.amount != INHERIT_DECIMAL {
            child.monthly_contribution.clone()
        } else {
            parent.monthly_contribution.clone()
        },
        holdings: if !child.holdings.is_empty() {
            child.holdings.clone()
        } else {
            parent.holdings.clone()
        },
    }
}

fn inherit_decimal() -> Decimal {
    INHERIT_DECIMAL
}

fn deserialize_rate<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_decimal(deserializer)
        .map_err(|_| de::Error::custom("rate must be a decimal string"))
}

#[cfg(test)]
mod tests;

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Decimal::from_str(&value).map_err(|_| de::Error::custom("value must be a decimal string"))
}

fn deserialize_optional_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|value| {
            Decimal::from_str(&value)
                .map_err(|_| de::Error::custom("value must be a decimal string"))
        })
        .transpose()
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Month {
    year: u16,
    month: u8,
}

impl Month {
    pub fn year(self) -> u16 {
        self.year
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn next(self) -> Self {
        if self.month == 12 {
            Self {
                year: self
                    .year
                    .checked_add(1)
                    .expect("month exceeds supported year range"),
                month: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month + 1,
            }
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04}-{:02}", self.year, self.month)
    }
}

impl Serialize for Month {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read configuration file `{}`: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse configuration file `{}`: {}", path.display(), source.message())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("plan end month {end} is before start month {start}")]
    InvalidPlanRange { start: Month, end: Month },
    #[error("conversion rate `{from}` → `{to}` must be greater than zero")]
    InvalidConversionRate { from: Currency, to: Currency },
    #[error(
        "conversion rate `{from}` → `{to}` must have exactly one endpoint matching plan currency `{plan_currency}`"
    )]
    InvalidConversionRateCurrencies {
        from: Currency,
        to: Currency,
        plan_currency: Currency,
    },
    #[error("multiple conversion rates connect `{from}` and `{to}`")]
    DuplicateConversionRate { from: Currency, to: Currency },
    #[error(
        "asset `{asset_id}` in scenario `{scenario_id}` uses `{currency}`, but no conversion rate connects it to plan currency `{plan_currency}`"
    )]
    MissingAssetConversionRate {
        scenario_id: String,
        asset_id: String,
        currency: Currency,
        plan_currency: Currency,
    },
    #[error(
        "holding `{holding_id}` in asset `{asset_id}` in scenario `{scenario_id}` uses `{currency}`, but no conversion rate connects it to plan currency `{plan_currency}`"
    )]
    MissingHoldingConversionRate {
        scenario_id: String,
        asset_id: String,
        holding_id: String,
        currency: Currency,
        plan_currency: Currency,
    },
    #[error("configuration must define at least one scenario")]
    NoScenarios,
    #[error("scenario id must not be blank")]
    BlankScenarioId,
    #[error("only one scenario may be selected")]
    MultipleSelectedScenarios,
    #[error("scenario `{id}` name must not be blank")]
    BlankScenarioName { id: String },
    #[error("scenario `{id}` name must be at most {maximum} characters")]
    ScenarioNameTooLong { id: String, maximum: usize },
    #[error("scenario `{id}` annual inflation must be greater than -1")]
    InvalidAnnualInflation { id: String },
    #[error("duplicate scenario id `{id}`")]
    DuplicateScenarioId { id: String },
    #[error("scenario `{id}` extends unknown scenario `{parent_id}`")]
    UnknownScenarioParent { id: String, parent_id: String },
    #[error("scenario `{id}` cannot extend itself")]
    ScenarioSelfExtension { id: String },
    #[error("scenario extension cycle includes `{id}`")]
    ScenarioExtensionCycle { id: String },
    #[error("scenario `{scenario_id}` must define at least one asset")]
    NoScenarioAssets { scenario_id: String },
    #[error("scenario `{scenario_id}` has an asset with a blank id")]
    BlankAssetId { scenario_id: String },
    #[error("asset `{asset_id}` in scenario `{scenario_id}` has a blank name")]
    BlankAssetName {
        scenario_id: String,
        asset_id: String,
    },
    #[error("duplicate asset id `{asset_id}` in scenario `{scenario_id}`")]
    DuplicateAssetId {
        scenario_id: String,
        asset_id: String,
    },
    #[error("asset `{asset_id}` in scenario `{scenario_id}` has a negative `{field}`")]
    NegativeAssetValue {
        scenario_id: String,
        asset_id: String,
        field: &'static str,
    },
    #[error("asset `{asset_id}` in scenario `{scenario_id}` must define at least one holding")]
    NoAssetHoldings {
        scenario_id: String,
        asset_id: String,
    },
    #[error("asset `{asset_id}` in scenario `{scenario_id}` has a holding with a blank id")]
    BlankHoldingId {
        scenario_id: String,
        asset_id: String,
    },
    #[error(
        "holding `{holding_id}` in asset `{asset_id}` in scenario `{scenario_id}` has a blank name"
    )]
    BlankHoldingName {
        scenario_id: String,
        asset_id: String,
        holding_id: String,
    },
    #[error(
        "duplicate holding id `{holding_id}` in asset `{asset_id}` in scenario `{scenario_id}`"
    )]
    DuplicateHoldingId {
        scenario_id: String,
        asset_id: String,
        holding_id: String,
    },
    #[error(
        "holding `{holding_id}` in asset `{asset_id}` in scenario `{scenario_id}` has a negative value"
    )]
    NegativeHoldingValue {
        scenario_id: String,
        asset_id: String,
        holding_id: String,
    },
    #[error(
        "contribution currency `{contribution_currency}` for asset `{asset_id}` in scenario `{scenario_id}` must be the asset currency `{asset_currency}` or plan currency `{plan_currency}"
    )]
    InvalidContributionCurrency {
        scenario_id: String,
        asset_id: String,
        contribution_currency: Currency,
        asset_currency: Currency,
        plan_currency: Currency,
    },
    #[error("scenario `{scenario_id}` has an event with a blank id")]
    BlankEventId { scenario_id: String },
    #[error("event `{event_id}` in scenario `{scenario_id}` has a blank name")]
    BlankEventName {
        scenario_id: String,
        event_id: String,
    },
    #[error("duplicate event id `{event_id}` in scenario `{scenario_id}`")]
    DuplicateEventId {
        scenario_id: String,
        event_id: String,
    },
    #[error(
        "event `{event_id}` in scenario `{scenario_id}` is dated {date}, outside the plan range {start} through {end}"
    )]
    EventOutsidePlan {
        scenario_id: String,
        event_id: String,
        date: Month,
        start: Month,
        end: Month,
    },
    #[error("event `{event_id}` in scenario `{scenario_id}` targets unknown asset `{asset_id}`")]
    UnknownEventAsset {
        scenario_id: String,
        event_id: String,
        asset_id: String,
    },
    #[error("event `{event_id}` in scenario `{scenario_id}` has a negative `{field}`")]
    NegativeEventValue {
        scenario_id: String,
        event_id: String,
        field: &'static str,
    },
    #[error("configuration has a total-balance milestone with a blank id")]
    BlankTotalBalanceMilestoneId,
    #[error("total-balance milestone `{id}` has a blank name")]
    BlankTotalBalanceMilestoneName { id: String },
    #[error("duplicate total-balance milestone id `{id}`")]
    DuplicateTotalBalanceMilestoneId { id: String },
    #[error("total-balance milestone `{id}` must have a positive target")]
    NonpositiveTotalBalanceMilestoneTarget { id: String },
    #[error("configuration has a future living cost with a blank id")]
    BlankFutureLivingCostId,
    #[error("future living cost `{id}` has a blank name")]
    BlankFutureLivingCostName { id: String },
    #[error("duplicate future living cost id `{id}`")]
    DuplicateFutureLivingCostId { id: String },
    #[error("future living cost `{id}` must have a positive monthly cost")]
    NonpositiveFutureLivingCost { id: String },
    #[error("future living cost `{id}` annual inflation must be greater than -1")]
    InvalidFutureLivingCostInflation { id: String },
    #[error("scenario `{scenario_id}` has a milestone with a blank id")]
    BlankMilestoneId { scenario_id: String },
    #[error("milestone `{milestone_id}` in scenario `{scenario_id}` has a blank name")]
    BlankMilestoneName {
        scenario_id: String,
        milestone_id: String,
    },
    #[error("duplicate milestone id `{milestone_id}` in scenario `{scenario_id}`")]
    DuplicateMilestoneId {
        scenario_id: String,
        milestone_id: String,
    },
    #[error("milestone `{milestone_id}` in scenario `{scenario_id}` must have a positive target")]
    NonpositiveMilestoneTarget {
        scenario_id: String,
        milestone_id: String,
    },
    #[error(
        "milestone `{milestone_id}` in scenario `{scenario_id}` targets unknown asset `{asset_id}`"
    )]
    UnknownMilestoneAsset {
        scenario_id: String,
        milestone_id: String,
        asset_id: String,
    },
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let mut config: Self = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&mut self) -> Result<(), ConfigError> {
        self.resolve_scenario_inheritance()?;
        self.plan.validate()?;
        self.validate_conversion_rates()?;
        self.validate_total_balance_milestones()?;
        self.validate_future_living_costs()?;
        self.validate_scenarios()
    }

    fn resolve_scenario_inheritance(&mut self) -> Result<(), ConfigError> {
        let scenarios = self.scenarios.clone();
        let mut resolved: Vec<Option<Scenario>> = vec![None; scenarios.len()];

        while resolved.iter().any(Option::is_none) {
            let mut made_progress = false;
            for (index, scenario) in scenarios.iter().enumerate() {
                if resolved[index].is_some() {
                    continue;
                }
                let parent = match scenario.extends.as_deref() {
                    None => None,
                    Some(parent_id) => {
                        let Some(parent_index) = scenarios
                            .iter()
                            .position(|candidate| candidate.id == parent_id)
                        else {
                            return Err(ConfigError::UnknownScenarioParent {
                                id: scenario.id.clone(),
                                parent_id: parent_id.to_owned(),
                            });
                        };
                        let Some(parent) = resolved[parent_index].as_ref() else {
                            continue;
                        };
                        Some(parent)
                    }
                };

                let mut scenario = scenario.clone();
                if let Some(parent) = parent {
                    if scenario.annual_inflation == INHERIT_DECIMAL {
                        scenario.annual_inflation = parent.annual_inflation;
                    }
                    let mut assets = parent.assets.clone();
                    for child_asset in &scenario.assets {
                        if let Some(index) =
                            assets.iter().position(|asset| asset.id == child_asset.id)
                        {
                            assets[index] = merge_asset(&assets[index], child_asset);
                        } else {
                            assets.push(child_asset.clone());
                        }
                    }
                    scenario.assets = assets;
                    let mut milestones = parent.milestones.clone();
                    milestones.extend(scenario.milestones);
                    scenario.milestones = milestones;
                }
                resolved[index] = Some(scenario);
                made_progress = true;
            }
            if !made_progress {
                let scenario = scenarios
                    .iter()
                    .zip(&resolved)
                    .find(|(_, resolved)| resolved.is_none())
                    .expect("unresolved scenario exists")
                    .0;
                return Err(ConfigError::ScenarioExtensionCycle {
                    id: scenario.id.clone(),
                });
            }
        }

        self.scenarios = resolved
            .into_iter()
            .map(|scenario| scenario.expect("all scenarios resolve"))
            .collect();
        Ok(())
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

            let non_plan_currency = if conversion_rate.from == self.plan.currency {
                &conversion_rate.to
            } else {
                &conversion_rate.from
            };
            if !seen.insert(non_plan_currency) {
                return Err(ConfigError::DuplicateConversionRate {
                    from: conversion_rate.from.clone(),
                    to: conversion_rate.to.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn conversion_rate_to_plan_currency(&self, currency: &Currency) -> Option<Decimal> {
        if currency == &self.plan.currency {
            return Some(Decimal::ONE);
        }

        self.conversion_rates
            .iter()
            .find(|rate| rate.from == *currency && rate.to == self.plan.currency)
            .map(|rate| rate.rate)
            .or_else(|| {
                self.conversion_rates
                    .iter()
                    .find(|rate| rate.from == self.plan.currency && rate.to == *currency)
                    .map(|rate| Decimal::ONE / rate.rate)
            })
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
            if cost
                .annual_inflation
                .is_some_and(|inflation| inflation <= -Decimal::ONE)
            {
                return Err(ConfigError::InvalidFutureLivingCostInflation {
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
        let mut has_selected_scenario = false;
        for scenario in &self.scenarios {
            if scenario.id.trim().is_empty() {
                return Err(ConfigError::BlankScenarioId);
            }
            if scenario.name.trim().is_empty() {
                return Err(ConfigError::BlankScenarioName {
                    id: scenario.id.clone(),
                });
            }
            if scenario.name.chars().count() > MAX_SCENARIO_NAME_LENGTH {
                return Err(ConfigError::ScenarioNameTooLong {
                    id: scenario.id.clone(),
                    maximum: MAX_SCENARIO_NAME_LENGTH,
                });
            }
            if scenario.annual_inflation <= -Decimal::ONE {
                return Err(ConfigError::InvalidAnnualInflation {
                    id: scenario.id.clone(),
                });
            }
            if scenario.selected && has_selected_scenario {
                return Err(ConfigError::MultipleSelectedScenarios);
            }
            has_selected_scenario |= scenario.selected;
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

            if self
                .conversion_rate_to_plan_currency(&asset.currency)
                .is_none()
            {
                return Err(ConfigError::MissingAssetConversionRate {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                    currency: asset.currency.clone(),
                    plan_currency: self.plan.currency.clone(),
                });
            }

            for (field, value) in [
                ("annual_expected_return", asset.annual_expected_return),
                (
                    "monthly_contribution.amount",
                    asset.monthly_contribution.amount,
                ),
            ] {
                if value < Decimal::ZERO {
                    return Err(ConfigError::NegativeAssetValue {
                        scenario_id: scenario.id.clone(),
                        asset_id: asset.id.clone(),
                        field,
                    });
                }
            }
            self.validate_contribution_currency(
                scenario,
                &asset.id,
                &asset.currency,
                &asset.monthly_contribution.currency,
            )?;
            self.validate_holdings(scenario, asset)?;
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
            if event.name().trim().is_empty() {
                return Err(ConfigError::BlankEventName {
                    scenario_id: scenario.id.clone(),
                    event_id: event.id().to_owned(),
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
                | Event::SetAnnualExpectedReturn { asset_id, .. } => asset_id,
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
            if let Event::SetMonthlyContribution { currency, .. } = event {
                let asset = self
                    .scenario_asset(scenario, asset_id, scenario_by_id)
                    .expect("validated event asset exists");
                self.validate_contribution_currency(scenario, asset_id, &asset.currency, currency)?;
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

    fn validate_holdings(&self, scenario: &Scenario, asset: &Asset) -> Result<(), ConfigError> {
        if asset.holdings.is_empty() {
            return Err(ConfigError::NoAssetHoldings {
                scenario_id: scenario.id.clone(),
                asset_id: asset.id.clone(),
            });
        }
        let mut holding_ids = HashSet::new();
        for holding in &asset.holdings {
            if holding.id.trim().is_empty() {
                return Err(ConfigError::BlankHoldingId {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                });
            }
            if holding.name.trim().is_empty() {
                return Err(ConfigError::BlankHoldingName {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                    holding_id: holding.id.clone(),
                });
            }
            if !holding_ids.insert(holding.id.as_str()) {
                return Err(ConfigError::DuplicateHoldingId {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                    holding_id: holding.id.clone(),
                });
            }
            if holding.value < Decimal::ZERO {
                return Err(ConfigError::NegativeHoldingValue {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                    holding_id: holding.id.clone(),
                });
            }
            if self
                .conversion_rate_to_plan_currency(&holding.currency)
                .is_none()
            {
                return Err(ConfigError::MissingHoldingConversionRate {
                    scenario_id: scenario.id.clone(),
                    asset_id: asset.id.clone(),
                    holding_id: holding.id.clone(),
                    currency: holding.currency.clone(),
                    plan_currency: self.plan.currency.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_contribution_currency(
        &self,
        scenario: &Scenario,
        asset_id: &str,
        asset_currency: &Currency,
        contribution_currency: &Currency,
    ) -> Result<(), ConfigError> {
        if contribution_currency == asset_currency || contribution_currency == &self.plan.currency {
            Ok(())
        } else {
            Err(ConfigError::InvalidContributionCurrency {
                scenario_id: scenario.id.clone(),
                asset_id: asset_id.to_owned(),
                contribution_currency: contribution_currency.clone(),
                asset_currency: asset_currency.clone(),
                plan_currency: self.plan.currency.clone(),
            })
        }
    }

    fn scenario_asset<'a>(
        &'a self,
        scenario: &'a Scenario,
        asset_id: &str,
        scenario_by_id: &HashMap<&str, &'a Scenario>,
    ) -> Option<&'a Asset> {
        let mut current = scenario;

        loop {
            if let Some(asset) = current.assets.iter().find(|asset| asset.id == asset_id) {
                return Some(asset);
            }

            let parent_id = current.extends.as_deref()?;
            current = scenario_by_id.get(parent_id)?;
        }
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
