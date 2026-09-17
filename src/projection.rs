use rust_decimal::{Decimal, MathematicalOps};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::config::{
    Asset, AssetMilestone, Config, ConversionRate, Currency, Event, FutureLivingCost, Month,
    MonthlyContribution, Plan, Scenario, TotalBalanceMilestone,
};

#[derive(Debug, Serialize)]
pub struct PlanProjection<'config> {
    pub plan: PlanContext<'config>,
    pub total_balance_milestones: Vec<TotalBalanceMilestoneProjection<'config>>,
    pub scenarios: Vec<ScenarioProjection<'config>>,
}

#[derive(Debug)]
pub struct PlanContext<'config> {
    plan: &'config Plan,
    conversion_rates: &'config [ConversionRate],
}

impl<'config> PlanContext<'config> {
    pub fn currency(&self) -> &'config Currency {
        &self.plan.currency
    }

    pub fn start(&self) -> Month {
        self.plan.start
    }

    pub fn end(&self) -> Month {
        self.plan.end
    }

    pub fn inclusive_month_count(&self) -> u32 {
        self.plan.inclusive_month_count()
    }

    pub fn conversion_rates(&self) -> &'config [ConversionRate] {
        self.conversion_rates
    }
}

impl Serialize for PlanContext<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PlanContext", 3)?;
        state.serialize_field("currency", &self.plan.currency)?;
        state.serialize_field("start", &self.plan.start)?;
        state.serialize_field("end", &self.plan.end)?;
        state.end()
    }
}

#[derive(Debug)]
pub struct ScenarioProjection<'config> {
    scenario: &'config Scenario,
    pub assets: Vec<AssetProjection<'config>>,
    pub total_net_worth: Vec<TotalNetWorthMonthProjection>,
    pub asset_adjustments: Vec<AssetAdjustmentProjection<'config>>,
    pub contribution_settings: Vec<ContributionSettingProjection<'config>>,
    pub asset_events: Vec<AppliedAssetEventProjection<'config>>,
    pub future_living_costs: FutureLivingCostsProjection<'config>,
    pub asset_milestones: Vec<AssetMilestoneProjection<'config>>,
}

impl<'config> ScenarioProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.scenario.id
    }

    pub fn name(&self) -> &'config str {
        &self.scenario.name
    }

    pub fn description(&self) -> Option<&'config str> {
        self.scenario.description.as_deref()
    }

    pub fn is_selected(&self) -> bool {
        self.scenario.selected
    }

    pub fn annual_inflation(&self) -> Decimal {
        self.scenario.annual_inflation
    }
}

impl Serialize for ScenarioProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ScenarioProjection", 11)?;
        state.serialize_field("id", &self.scenario.id)?;
        state.serialize_field("name", &self.scenario.name)?;
        state.serialize_field("description", &self.scenario.description)?;
        state.serialize_field("selected", &self.scenario.selected)?;
        state.serialize_field("assets", &self.assets)?;
        state.serialize_field("total_net_worth", &self.total_net_worth)?;
        state.serialize_field("asset_adjustments", &self.asset_adjustments)?;
        state.serialize_field("contribution_settings", &self.contribution_settings)?;
        state.serialize_field("asset_events", &self.asset_events)?;
        state.serialize_field("future_living_costs", &self.future_living_costs)?;
        state.serialize_field("asset_milestones", &self.asset_milestones)?;
        state.end()
    }
}

#[derive(Clone, Debug)]
pub struct AssetProjection<'config> {
    asset: &'config Asset,
    pub monthly_balances: Vec<AssetMonthProjection>,
}

impl<'config> AssetProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.asset.id
    }

    pub fn name(&self) -> &'config str {
        &self.asset.name
    }

    pub fn currency(&self) -> &'config Currency {
        &self.asset.currency
    }
}

impl Serialize for AssetProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetProjection", 4)?;
        state.serialize_field("id", self.id())?;
        state.serialize_field("name", self.name())?;
        state.serialize_field("currency", self.currency())?;
        state.serialize_field("monthly_balances", &self.monthly_balances)?;
        state.end()
    }
}

#[derive(Clone, Debug)]
pub struct AssetMonthProjection {
    pub month: Month,
    pub annual_expected_return: Decimal,
    pub monthly_contribution: MonthlyContribution,
    pub native_balance: Decimal,
    pub plan_balance: Decimal,
    pub native_passive_income: Decimal,
    pub plan_passive_income: Decimal,
}

impl Serialize for AssetMonthProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetMonthProjection", 7)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field(
            "annual_expected_return",
            &self.annual_expected_return.to_string(),
        )?;
        state.serialize_field("monthly_contribution", &self.monthly_contribution)?;
        state.serialize_field("native_balance", &self.native_balance.to_string())?;
        state.serialize_field("plan_balance", &self.plan_balance.to_string())?;
        state.serialize_field(
            "native_passive_income",
            &self.native_passive_income.to_string(),
        )?;
        state.serialize_field("plan_passive_income", &self.plan_passive_income.to_string())?;
        state.end()
    }
}

#[derive(Clone, Debug)]
pub struct TotalNetWorthMonthProjection {
    pub month: Month,
    pub balance: Decimal,
}

impl Serialize for TotalNetWorthMonthProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TotalNetWorthMonthProjection", 2)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field("balance", &self.balance.to_string())?;
        state.end()
    }
}

#[derive(Debug, Serialize)]
pub struct FutureLivingCostsProjection<'config> {
    pub costs: Vec<FutureLivingCostProjection<'config>>,
    pub nominal_monthly_total: PlanMoney<'config>,
}

#[derive(Debug)]
pub struct FutureLivingCostProjection<'config> {
    cost: &'config FutureLivingCost,
    pub annual_inflation: Decimal,
    pub nominal_monthly_cost: PlanMoney<'config>,
}

impl<'config> FutureLivingCostProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.cost.id
    }

    pub fn name(&self) -> &'config str {
        &self.cost.name
    }

    pub fn description(&self) -> Option<&'config str> {
        self.cost.description.as_deref()
    }

    pub fn today_money_monthly_cost(&self) -> Decimal {
        self.cost.monthly_cost
    }
}

impl Serialize for FutureLivingCostProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FutureLivingCostProjection", 5)?;
        state.serialize_field("id", &self.cost.id)?;
        state.serialize_field("name", &self.cost.name)?;
        state.serialize_field("description", &self.cost.description)?;
        state.serialize_field("annual_inflation", &self.annual_inflation.to_string())?;
        state.serialize_field("nominal_monthly_cost", &self.nominal_monthly_cost)?;
        state.end()
    }
}

#[derive(Debug)]
pub struct TotalBalanceMilestoneProjection<'config> {
    milestone: &'config TotalBalanceMilestone,
    pub target: PlanMoney<'config>,
}

impl<'config> TotalBalanceMilestoneProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.milestone.id
    }

    pub fn name(&self) -> &'config str {
        &self.milestone.name
    }
}

impl Serialize for TotalBalanceMilestoneProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TotalBalanceMilestoneProjection", 3)?;
        state.serialize_field("id", &self.milestone.id)?;
        state.serialize_field("name", &self.milestone.name)?;
        state.serialize_field("target", &self.target)?;
        state.end()
    }
}

#[derive(Debug)]
pub struct AssetMilestoneProjection<'config> {
    milestone: &'config AssetMilestone,
    pub target: PlanMoney<'config>,
}

impl<'config> AssetMilestoneProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.milestone.id
    }

    pub fn name(&self) -> &'config str {
        &self.milestone.name
    }

    pub fn asset_id(&self) -> &'config str {
        &self.milestone.asset_id
    }
}

impl Serialize for AssetMilestoneProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetMilestoneProjection", 3)?;
        state.serialize_field("id", &self.milestone.id)?;
        state.serialize_field("name", &self.milestone.name)?;
        state.serialize_field("target", &self.target)?;
        state.end()
    }
}

#[derive(Debug)]
pub enum PlanMoney<'config> {
    PlanCurrency {
        amount: PlanAmount<'config>,
    },
    Converted {
        // Retained for future hover details alongside the comparable plan amount.
        #[allow(dead_code)]
        original_amount: &'config Decimal,
        #[allow(dead_code)]
        original_currency: &'config Currency,
        plan_amount: Decimal,
    },
}

#[derive(Debug)]
pub enum PlanAmount<'config> {
    Configured(&'config Decimal),
    Calculated(Decimal),
}

impl PlanMoney<'_> {
    fn configured_in_plan_currency<'config>(amount: &'config Decimal) -> PlanMoney<'config> {
        PlanMoney::PlanCurrency {
            amount: PlanAmount::Configured(amount),
        }
    }

    fn calculated_in_plan_currency<'config>(amount: Decimal) -> PlanMoney<'config> {
        PlanMoney::PlanCurrency {
            amount: PlanAmount::Calculated(amount),
        }
    }

    pub(crate) fn plan_amount(&self) -> Decimal {
        match self {
            Self::PlanCurrency {
                amount: PlanAmount::Configured(amount),
            } => **amount,
            Self::PlanCurrency {
                amount: PlanAmount::Calculated(amount),
            }
            | Self::Converted {
                plan_amount: amount,
                ..
            } => *amount,
        }
    }

    #[allow(dead_code)]
    pub fn original_amount_and_currency(&self) -> Option<(&Decimal, &Currency)> {
        match self {
            Self::PlanCurrency { .. } => None,
            Self::Converted {
                original_amount,
                original_currency,
                ..
            } => Some((original_amount, original_currency)),
        }
    }
}

impl Serialize for PlanMoney<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.plan_amount().to_string())
    }
}

impl<'config> From<&'config Config> for PlanProjection<'config> {
    fn from(config: &'config Config) -> Self {
        let inflation_years =
            Decimal::from(config.plan.inclusive_month_count()) / Decimal::from(12);
        let total_balance_milestones = config
            .milestones
            .iter()
            .map(|milestone| TotalBalanceMilestoneProjection {
                milestone,
                target: PlanMoney::configured_in_plan_currency(&milestone.target),
            })
            .collect();
        let projected_asset_sets = project_scenario_assets(config);

        Self {
            plan: PlanContext {
                plan: &config.plan,
                conversion_rates: &config.conversion_rates,
            },
            total_balance_milestones,
            scenarios: config
                .scenarios
                .iter()
                .zip(projected_asset_sets)
                .map(|(scenario, assets)| {
                    let costs = config
                        .future_living_costs
                        .iter()
                        .map(|cost| {
                            let annual_inflation =
                                cost.annual_inflation.unwrap_or(scenario.annual_inflation);
                            let inflation_factor =
                                (Decimal::ONE + annual_inflation).powd(inflation_years);
                            FutureLivingCostProjection {
                                cost,
                                annual_inflation,
                                nominal_monthly_cost: PlanMoney::calculated_in_plan_currency(
                                    cost.monthly_cost * inflation_factor,
                                ),
                            }
                        })
                        .collect::<Vec<_>>();
                    let nominal_monthly_total = PlanMoney::calculated_in_plan_currency(
                        costs
                            .iter()
                            .map(|cost| cost.nominal_monthly_cost.plan_amount())
                            .sum(),
                    );
                    let asset_milestones = scenario
                        .milestones
                        .iter()
                        .map(|milestone| {
                            let currency = asset_currency(config, scenario, &milestone.asset_id);
                            let target = if currency == &config.plan.currency {
                                PlanMoney::configured_in_plan_currency(&milestone.target)
                            } else {
                                PlanMoney::Converted {
                                    original_amount: &milestone.target,
                                    original_currency: currency,
                                    plan_amount: milestone.target
                                        * config.conversion_rate_to_plan_currency(currency).expect(
                                            "validated asset currencies have a conversion rate",
                                        ),
                                }
                            };

                            AssetMilestoneProjection { milestone, target }
                        })
                        .collect();

                    let total_net_worth = project_total_net_worth(&assets);
                    let asset_adjustments = resolved_asset_adjustments(config, scenario);
                    let contribution_settings = resolved_contribution_settings(config, scenario);
                    let asset_events = resolved_asset_events(config, scenario);

                    ScenarioProjection {
                        scenario,
                        assets,
                        total_net_worth,
                        asset_adjustments,
                        contribution_settings,
                        asset_events,
                        future_living_costs: FutureLivingCostsProjection {
                            costs,
                            nominal_monthly_total,
                        },
                        asset_milestones,
                    }
                })
                .collect(),
        }
    }
}

fn project_total_net_worth(assets: &[AssetProjection<'_>]) -> Vec<TotalNetWorthMonthProjection> {
    let Some(first_asset) = assets.first() else {
        return Vec::new();
    };

    first_asset
        .monthly_balances
        .iter()
        .enumerate()
        .map(|(index, balance)| TotalNetWorthMonthProjection {
            month: balance.month,
            balance: assets
                .iter()
                .map(|asset| asset.monthly_balances[index].plan_balance)
                .sum(),
        })
        .collect()
}

fn project_scenario_assets<'config>(config: &'config Config) -> Vec<Vec<AssetProjection<'config>>> {
    let mut projected_asset_sets = (0..config.scenarios.len())
        .map(|_| None)
        .collect::<Vec<Option<Vec<AssetProjection<'config>>>>>();

    while projected_asset_sets.iter().any(Option::is_none) {
        let mut made_progress = false;

        for (index, scenario) in config.scenarios.iter().enumerate() {
            if projected_asset_sets[index].is_some() {
                continue;
            }

            let mut assets = match scenario.extends.as_deref() {
                None => Vec::new(),
                Some(parent_id) => {
                    let parent_index = config
                        .scenarios
                        .iter()
                        .position(|candidate| candidate.id == parent_id)
                        .expect("validated scenario parents always exist");
                    let Some(parent_assets) = projected_asset_sets[parent_index].as_ref() else {
                        continue;
                    };
                    parent_assets.clone()
                }
            };

            for asset in &scenario.assets {
                let projection = AssetProjection {
                    asset,
                    monthly_balances: Vec::new(),
                };
                if let Some(existing_index) = assets
                    .iter()
                    .position(|inherited_asset| inherited_asset.id() == asset.id)
                {
                    assets[existing_index] = projection;
                } else {
                    assets.push(projection);
                }
            }

            let adjustments = resolved_asset_adjustments(config, scenario);
            let contribution_settings = resolved_contribution_settings(config, scenario);
            let return_settings = resolved_return_settings(config, scenario);
            for asset in &mut assets {
                *asset = project_asset(
                    config,
                    asset.asset,
                    &adjustments,
                    &contribution_settings,
                    &return_settings,
                );
            }

            projected_asset_sets[index] = Some(assets);
            made_progress = true;
        }

        assert!(
            made_progress,
            "validated scenario inheritance must allow asset projection ordering"
        );
    }

    projected_asset_sets
        .into_iter()
        .map(|assets| assets.expect("all scenarios receive projected assets"))
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct AssetAdjustmentProjection<'config> {
    pub name: &'config str,
    pub date: Month,
    pub asset_id: &'config str,
    pub amount: &'config Decimal,
}

impl Serialize for AssetAdjustmentProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetAdjustmentProjection", 4)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("asset_id", self.asset_id)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.end()
    }
}

fn scenario_lineage<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<&'config Scenario> {
    let mut lineage = Vec::new();
    let mut current = scenario;
    loop {
        lineage.push(current);
        let Some(parent_id) = current.extends.as_deref() else {
            break;
        };
        current = config
            .scenarios
            .iter()
            .find(|candidate| candidate.id == parent_id)
            .expect("validated scenario parents always exist");
    }
    lineage.reverse();
    lineage
}

fn resolved_asset_adjustments<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<AssetAdjustmentProjection<'config>> {
    scenario_lineage(config, scenario)
        .into_iter()
        .flat_map(|scenario| scenario.events.iter())
        .filter_map(|event| match event {
            Event::AssetAdjustment {
                name,
                date,
                asset_id,
                amount,
                ..
            } => Some(AssetAdjustmentProjection {
                name,
                date: *date,
                asset_id,
                amount,
            }),
            Event::SetMonthlyContribution { .. } | Event::SetAnnualExpectedReturn { .. } => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedAssetEventKind {
    Adjustment,
    ContributionSetting,
    ExpectedReturn,
}

#[derive(Clone, Copy, Debug)]
pub struct AppliedAssetEventProjection<'config> {
    pub name: &'config str,
    pub date: Month,
    pub asset_id: &'config str,
    pub kind: AppliedAssetEventKind,
    pub amount: &'config Decimal,
    pub currency: Option<&'config Currency>,
}

impl Serialize for AppliedAssetEventProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppliedAssetEventProjection", 6)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("asset_id", self.asset_id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.serialize_field("currency", &self.currency)?;
        state.end()
    }
}

fn resolved_asset_events<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<AppliedAssetEventProjection<'config>> {
    scenario_lineage(config, scenario)
        .into_iter()
        .flat_map(|scenario| scenario.events.iter())
        .map(|event| match event {
            Event::AssetAdjustment {
                name,
                date,
                asset_id,
                amount,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: *date,
                asset_id,
                kind: AppliedAssetEventKind::Adjustment,
                amount,
                currency: None,
            },
            Event::SetMonthlyContribution {
                name,
                date,
                asset_id,
                amount,
                currency,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: *date,
                asset_id,
                kind: AppliedAssetEventKind::ContributionSetting,
                amount,
                currency: Some(currency),
            },
            Event::SetAnnualExpectedReturn {
                name,
                date,
                asset_id,
                rate,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: *date,
                asset_id,
                kind: AppliedAssetEventKind::ExpectedReturn,
                amount: rate,
                currency: None,
            },
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct ContributionSettingProjection<'config> {
    pub name: &'config str,
    pub date: Month,
    pub asset_id: &'config str,
    pub amount: &'config Decimal,
    pub currency: &'config Currency,
}

impl Serialize for ContributionSettingProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ContributionSettingProjection", 5)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("asset_id", self.asset_id)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.serialize_field("currency", self.currency)?;
        state.end()
    }
}

fn resolved_contribution_settings<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<ContributionSettingProjection<'config>> {
    scenario_lineage(config, scenario)
        .into_iter()
        .flat_map(|scenario| scenario.events.iter())
        .filter_map(|event| match event {
            Event::SetMonthlyContribution {
                name,
                date,
                asset_id,
                amount,
                currency,
                ..
            } => Some(ContributionSettingProjection {
                name,
                date: *date,
                asset_id,
                amount,
                currency,
            }),
            Event::AssetAdjustment { .. } | Event::SetAnnualExpectedReturn { .. } => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct ResolvedReturnSetting<'config> {
    date: Month,
    asset_id: &'config str,
    rate: &'config Decimal,
}

fn resolved_return_settings<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<ResolvedReturnSetting<'config>> {
    scenario_lineage(config, scenario)
        .into_iter()
        .flat_map(|scenario| scenario.events.iter())
        .filter_map(|event| match event {
            Event::SetAnnualExpectedReturn {
                date,
                asset_id,
                rate,
                ..
            } => Some(ResolvedReturnSetting {
                date: *date,
                asset_id,
                rate,
            }),
            Event::AssetAdjustment { .. } | Event::SetMonthlyContribution { .. } => None,
        })
        .collect()
}

fn monthly_rate(annual_rate: Decimal) -> Decimal {
    (Decimal::ONE + annual_rate).powd(Decimal::ONE / Decimal::from(12)) - Decimal::ONE
}

fn project_asset<'config>(
    config: &'config Config,
    asset: &'config Asset,
    adjustments: &[AssetAdjustmentProjection<'config>],
    contribution_settings: &[ContributionSettingProjection<'config>],
    return_settings: &[ResolvedReturnSetting<'config>],
) -> AssetProjection<'config> {
    let conversion_rate = config
        .conversion_rate_to_plan_currency(&asset.currency)
        .expect("validated asset currencies have a conversion rate");
    let mut month = config.plan.start;
    let mut native_balance = asset
        .holdings
        .iter()
        .map(|holding| {
            holding.value
                * config
                    .conversion_rate_to_plan_currency(&holding.currency)
                    .expect("validated holding currencies have a conversion rate")
                / config
                    .conversion_rate_to_plan_currency(&asset.currency)
                    .expect("validated asset currencies have a conversion rate")
        })
        .sum();
    let mut annual_expected_return = asset.annual_expected_return;
    let mut monthly_contribution = asset.monthly_contribution.clone();
    let month_count = config.plan.inclusive_month_count();
    let monthly_balances = (0..month_count)
        .map(|index| {
            if let Some(setting) = return_settings
                .iter()
                .rev()
                .find(|setting| setting.date == month && setting.asset_id == asset.id)
            {
                annual_expected_return = *setting.rate;
            }
            if let Some(setting) = contribution_settings
                .iter()
                .rev()
                .find(|setting| setting.date == month && setting.asset_id == asset.id)
            {
                monthly_contribution = MonthlyContribution {
                    amount: *setting.amount,
                    currency: setting.currency.clone(),
                };
            }
            let adjustment_total = adjustments
                .iter()
                .filter(|adjustment| adjustment.date == month && adjustment.asset_id == asset.id)
                .map(|adjustment| *adjustment.amount)
                .sum::<Decimal>();
            let monthly_expected_return = monthly_rate(annual_expected_return);
            let native_passive_income = native_balance * monthly_expected_return;
            let plan_passive_income = native_passive_income * conversion_rate;
            let native_monthly_contribution = if monthly_contribution.currency == asset.currency {
                monthly_contribution.amount
            } else {
                monthly_contribution.amount / conversion_rate
            };
            native_balance = native_balance * (Decimal::ONE + monthly_expected_return)
                + native_monthly_contribution
                + adjustment_total;
            let projection = AssetMonthProjection {
                month,
                annual_expected_return,
                monthly_contribution: monthly_contribution.clone(),
                native_balance,
                plan_balance: native_balance * conversion_rate,
                native_passive_income,
                plan_passive_income,
            };
            if index + 1 < month_count {
                month = month.next();
            }
            projection
        })
        .collect();

    AssetProjection {
        asset,
        monthly_balances,
    }
}

fn asset_currency<'a>(config: &'a Config, scenario: &'a Scenario, asset_id: &str) -> &'a Currency {
    let mut current = scenario;

    loop {
        if let Some(asset) = current.assets.iter().find(|asset| asset.id == asset_id) {
            return &asset.currency;
        }
        current = config
            .scenarios
            .iter()
            .find(|candidate| Some(&candidate.id) == current.extends.as_ref())
            .expect("validated milestone assets always resolve through a scenario parent");
    }
}

#[cfg(test)]
mod tests;
