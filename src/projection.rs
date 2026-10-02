use rust_decimal::{Decimal, MathematicalOps};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::config::{
    Asset, AssetMilestone, Config, ConversionRate, Currency, Event, FutureLivingCost, Month,
    MonthlyContribution, MonthlyIncome, MonthlyWithdrawal, Plan, RecurringFlowAdjustment, Scenario,
    TotalBalanceMilestone, resolved_event_occurrences,
};

#[derive(Debug, Serialize)]
pub struct PlanProjection<'config> {
    pub plan: PlanContext<'config>,
    pub total_balance_milestones: Vec<TotalBalanceMilestoneProjection<'config>>,
    pub scenarios: Vec<ScenarioProjection<'config>>,
}

#[derive(Debug)]
pub struct ActualBalanceComparisons<'config> {
    pub scenarios: Vec<ScenarioActualBalanceComparison<'config>>,
}

#[derive(Debug)]
pub struct ScenarioActualBalanceComparison<'config> {
    pub scenario_id: &'config str,
    pub months: Vec<MonthActualBalanceComparison<'config>>,
}

#[derive(Debug)]
pub struct MonthActualBalanceComparison<'config> {
    pub month: Month,
    pub note: Option<&'config str>,
    pub assets: Vec<AssetActualBalanceComparison<'config>>,
    pub total: Option<TotalActualBalanceComparison>,
}

#[derive(Debug)]
pub struct AssetActualBalanceComparison<'config> {
    pub asset_id: &'config str,
    pub planned_native_balance: Decimal,
    pub actual_native_balance: Decimal,
    pub native_difference: Decimal,
    pub planned_plan_balance: Decimal,
    pub actual_plan_balance: Decimal,
    pub plan_difference: Decimal,
}

#[derive(Debug)]
pub struct TotalActualBalanceComparison {
    pub planned_balance: Decimal,
    pub actual_balance: Decimal,
    pub difference: Decimal,
}

impl<'config> ActualBalanceComparisons<'config> {
    pub fn new(config: &'config Config, projection: &'config PlanProjection<'config>) -> Self {
        Self {
            scenarios: projection
                .scenarios
                .iter()
                .map(|scenario| ScenarioActualBalanceComparison {
                    scenario_id: scenario.id(),
                    months: scenario
                        .total_net_worth
                        .iter()
                        .enumerate()
                        .map(|(index, total)| {
                            let actual_balances = config
                                .actual_months
                                .get(&total.month)
                                .map(|actual| &actual.balances);
                            let active_assets = scenario
                                .assets
                                .iter()
                                .filter_map(|asset| {
                                    let planned = &asset.monthly_balances[index];
                                    if !planned.is_active {
                                        return None;
                                    }
                                    let actual = actual_balances?.get(asset.id())?.0;
                                    let actual_plan_balance = actual
                                        * config
                                            .conversion_rate_to_plan_currency(asset.currency())
                                            .expect(
                                                "validated asset currencies have a conversion rate",
                                            );
                                    Some(AssetActualBalanceComparison {
                                        asset_id: asset.id(),
                                        planned_native_balance: planned.pre_actual_native_balance,
                                        actual_native_balance: actual,
                                        native_difference: actual
                                            - planned.pre_actual_native_balance,
                                        planned_plan_balance: planned.pre_actual_plan_balance,
                                        actual_plan_balance,
                                        plan_difference: actual_plan_balance
                                            - planned.pre_actual_plan_balance,
                                    })
                                })
                                .collect::<Vec<_>>();
                            let active_asset_count = scenario
                                .assets
                                .iter()
                                .filter(|asset| asset.monthly_balances[index].is_active)
                                .count();
                            let actual_total = (active_asset_count > 0
                                && active_assets.len() == active_asset_count)
                                .then(|| {
                                    let actual_balance = active_assets
                                        .iter()
                                        .map(|asset| asset.actual_plan_balance)
                                        .sum();
                                    let planned_balance = scenario
                                        .assets
                                        .iter()
                                        .map(|asset| {
                                            asset.monthly_balances[index].pre_actual_plan_balance
                                        })
                                        .sum();
                                    TotalActualBalanceComparison {
                                        planned_balance,
                                        actual_balance,
                                        difference: actual_balance - planned_balance,
                                    }
                                });

                            MonthActualBalanceComparison {
                                month: total.month,
                                note: config
                                    .actual_months
                                    .get(&total.month)
                                    .and_then(|actual| actual.note.as_deref()),
                                assets: active_assets,
                                total: actual_total,
                            }
                        })
                        .collect(),
                })
                .collect(),
        }
    }
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

    pub fn withdrawal_rate(&self) -> Decimal {
        self.plan.withdrawal_rate
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
    pub withdrawal_settings: Vec<WithdrawalSettingProjection<'config>>,
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

    pub fn monthly_income(&self) -> Option<&'config MonthlyIncome> {
        self.scenario.monthly_income.as_ref()
    }
}

impl Serialize for ScenarioProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ScenarioProjection", 12)?;
        state.serialize_field("id", &self.scenario.id)?;
        state.serialize_field("name", &self.scenario.name)?;
        state.serialize_field("description", &self.scenario.description)?;
        state.serialize_field("selected", &self.scenario.selected)?;
        state.serialize_field("assets", &self.assets)?;
        state.serialize_field("total_net_worth", &self.total_net_worth)?;
        state.serialize_field("asset_adjustments", &self.asset_adjustments)?;
        state.serialize_field("contribution_settings", &self.contribution_settings)?;
        state.serialize_field("withdrawal_settings", &self.withdrawal_settings)?;
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
    pub is_active: bool,
    pub annual_expected_return: Decimal,
    pub monthly_contribution: MonthlyContribution,
    pub monthly_withdrawal: MonthlyWithdrawal,
    pub native_balance: Decimal,
    pub plan_balance: Decimal,
    /// End-of-month forecast before an observed balance replaces it.
    pub pre_actual_native_balance: Decimal,
    pub pre_actual_plan_balance: Decimal,
    pub native_passive_income: Decimal,
    pub plan_passive_income: Decimal,
}

impl Serialize for AssetMonthProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetMonthProjection", 8)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field("is_active", &self.is_active)?;
        state.serialize_field(
            "annual_expected_return",
            &self.annual_expected_return.to_string(),
        )?;
        state.serialize_field("monthly_contribution", &self.monthly_contribution)?;
        state.serialize_field("monthly_withdrawal", &self.monthly_withdrawal)?;
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
    pub monthly_investment_rate: Option<Decimal>,
}

impl Serialize for TotalNetWorthMonthProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TotalNetWorthMonthProjection", 3)?;
        state.serialize_field("month", &self.month)?;
        state.serialize_field("balance", &self.balance.to_string())?;
        state.serialize_field(
            "monthly_investment_rate",
            &self.monthly_investment_rate.map(|rate| rate.to_string()),
        )?;
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

                    let total_net_worth = project_total_net_worth(config, scenario, &assets);
                    let asset_adjustments = resolved_asset_adjustments(config, scenario);
                    let contribution_settings = resolved_contribution_settings(config, scenario);
                    let withdrawal_settings = resolved_withdrawal_settings(config, scenario);
                    let asset_events = resolved_asset_events(config, scenario);

                    ScenarioProjection {
                        scenario,
                        assets,
                        total_net_worth,
                        asset_adjustments,
                        contribution_settings,
                        withdrawal_settings,
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

fn project_total_net_worth(
    config: &Config,
    scenario: &Scenario,
    assets: &[AssetProjection<'_>],
) -> Vec<TotalNetWorthMonthProjection> {
    let Some(first_asset) = assets.first() else {
        return Vec::new();
    };

    first_asset
        .monthly_balances
        .iter()
        .enumerate()
        .map(|(index, balance)| {
            let monthly_investment_rate = scenario.monthly_income.as_ref().map(|income| {
                assets
                    .iter()
                    .filter_map(|asset| {
                        let month = &asset.monthly_balances[index];
                        month.is_active.then(|| {
                            plan_monthly_flow_amount(
                                config,
                                month.monthly_contribution.amount,
                                &month.monthly_contribution.currency,
                            ) - plan_monthly_flow_amount(
                                config,
                                month.monthly_withdrawal.amount,
                                &month.monthly_withdrawal.currency,
                            )
                        })
                    })
                    .sum::<Decimal>()
                    / (income.amount
                        * config
                            .conversion_rate_to_plan_currency(&income.currency)
                            .expect("validated income currencies have a conversion rate"))
            });
            TotalNetWorthMonthProjection {
                month: balance.month,
                balance: assets
                    .iter()
                    .map(|asset| asset.monthly_balances[index].plan_balance)
                    .sum(),
                monthly_investment_rate,
            }
        })
        .collect()
}

fn plan_monthly_flow_amount(config: &Config, amount: Decimal, currency: &Currency) -> Decimal {
    amount
        * config
            .conversion_rate_to_plan_currency(currency)
            .expect("validated recurring-flow currencies have a conversion rate")
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
            let withdrawal_settings = resolved_withdrawal_settings(config, scenario);
            let return_settings = resolved_return_settings(config, scenario);
            for asset in &mut assets {
                *asset = project_asset(
                    config,
                    asset.asset,
                    &adjustments,
                    &contribution_settings,
                    &withdrawal_settings,
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

fn resolved_asset_adjustments<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<AssetAdjustmentProjection<'config>> {
    resolved_event_occurrences(config, scenario)
        .into_iter()
        .filter_map(|occurrence| match occurrence.event {
            Event::AssetAdjustment {
                name,
                asset_id,
                amount,
                ..
            } => Some(AssetAdjustmentProjection {
                name,
                date: occurrence.date,
                asset_id,
                amount,
            }),
            Event::SetMonthlyContribution { .. }
            | Event::SetMonthlyWithdrawal { .. }
            | Event::AdjustMonthlyContribution { .. }
            | Event::AdjustMonthlyWithdrawal { .. }
            | Event::SetAnnualExpectedReturn { .. } => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppliedAssetEventKind {
    Adjustment,
    ContributionSetting,
    WithdrawalSetting,
    ContributionAdjustment,
    WithdrawalAdjustment,
    ContributionRateAdjustment,
    WithdrawalRateAdjustment,
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
    resolved_event_occurrences(config, scenario)
        .into_iter()
        .map(|occurrence| match occurrence.event {
            Event::AssetAdjustment {
                name,
                asset_id,
                amount,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: occurrence.date,
                asset_id,
                kind: AppliedAssetEventKind::Adjustment,
                amount,
                currency: None,
            },
            Event::SetMonthlyContribution {
                name,
                asset_id,
                monthly_contribution,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: occurrence.date,
                asset_id,
                kind: AppliedAssetEventKind::ContributionSetting,
                amount: &monthly_contribution.amount,
                currency: Some(&monthly_contribution.currency),
            },
            Event::SetMonthlyWithdrawal {
                name,
                asset_id,
                monthly_withdrawal,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: occurrence.date,
                asset_id,
                kind: AppliedAssetEventKind::WithdrawalSetting,
                amount: &monthly_withdrawal.amount,
                currency: Some(&monthly_withdrawal.currency),
            },
            Event::AdjustMonthlyContribution {
                name,
                asset_id,
                monthly_contribution,
                ..
            } => {
                let (kind, amount, currency) = match monthly_contribution {
                    RecurringFlowAdjustment::Amount(adjustment) => (
                        AppliedAssetEventKind::ContributionAdjustment,
                        &adjustment.amount,
                        Some(&adjustment.currency),
                    ),
                    RecurringFlowAdjustment::Rate(adjustment) => (
                        AppliedAssetEventKind::ContributionRateAdjustment,
                        &adjustment.rate,
                        None,
                    ),
                };
                AppliedAssetEventProjection {
                    name,
                    date: occurrence.date,
                    asset_id,
                    kind,
                    amount,
                    currency,
                }
            }
            Event::AdjustMonthlyWithdrawal {
                name,
                asset_id,
                monthly_withdrawal,
                ..
            } => {
                let (kind, amount, currency) = match monthly_withdrawal {
                    RecurringFlowAdjustment::Amount(adjustment) => (
                        AppliedAssetEventKind::WithdrawalAdjustment,
                        &adjustment.amount,
                        Some(&adjustment.currency),
                    ),
                    RecurringFlowAdjustment::Rate(adjustment) => (
                        AppliedAssetEventKind::WithdrawalRateAdjustment,
                        &adjustment.rate,
                        None,
                    ),
                };
                AppliedAssetEventProjection {
                    name,
                    date: occurrence.date,
                    asset_id,
                    kind,
                    amount,
                    currency,
                }
            }
            Event::SetAnnualExpectedReturn {
                name,
                asset_id,
                rate,
                ..
            } => AppliedAssetEventProjection {
                name,
                date: occurrence.date,
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
    pub currency: Option<&'config Currency>,
    pub is_adjustment: bool,
    pub is_percentage: bool,
}

impl Serialize for ContributionSettingProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ContributionSettingProjection", 7)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("asset_id", self.asset_id)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.serialize_field("currency", &self.currency)?;
        state.serialize_field("is_adjustment", &self.is_adjustment)?;
        state.serialize_field("is_percentage", &self.is_percentage)?;
        state.end()
    }
}

fn resolved_contribution_settings<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<ContributionSettingProjection<'config>> {
    resolved_event_occurrences(config, scenario)
        .into_iter()
        .filter_map(|occurrence| match occurrence.event {
            Event::SetMonthlyContribution {
                name,
                asset_id,
                monthly_contribution,
                ..
            } => Some(ContributionSettingProjection {
                name,
                date: occurrence.date,
                asset_id,
                amount: &monthly_contribution.amount,
                currency: Some(&monthly_contribution.currency),
                is_adjustment: false,
                is_percentage: false,
            }),
            Event::AdjustMonthlyContribution {
                name,
                asset_id,
                monthly_contribution,
                ..
            } => {
                let (amount, currency, is_percentage) = match monthly_contribution {
                    RecurringFlowAdjustment::Amount(adjustment) => {
                        (&adjustment.amount, Some(&adjustment.currency), false)
                    }
                    RecurringFlowAdjustment::Rate(adjustment) => (&adjustment.rate, None, true),
                };
                Some(ContributionSettingProjection {
                    name,
                    date: occurrence.date,
                    asset_id,
                    amount,
                    currency,
                    is_adjustment: true,
                    is_percentage,
                })
            }
            Event::AssetAdjustment { .. }
            | Event::SetMonthlyWithdrawal { .. }
            | Event::AdjustMonthlyWithdrawal { .. }
            | Event::SetAnnualExpectedReturn { .. } => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct WithdrawalSettingProjection<'config> {
    pub name: &'config str,
    pub date: Month,
    pub asset_id: &'config str,
    pub amount: &'config Decimal,
    pub currency: Option<&'config Currency>,
    pub is_adjustment: bool,
    pub is_percentage: bool,
}

impl Serialize for WithdrawalSettingProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WithdrawalSettingProjection", 7)?;
        state.serialize_field("name", self.name)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("asset_id", self.asset_id)?;
        state.serialize_field("amount", &self.amount.to_string())?;
        state.serialize_field("currency", &self.currency)?;
        state.serialize_field("is_adjustment", &self.is_adjustment)?;
        state.serialize_field("is_percentage", &self.is_percentage)?;
        state.end()
    }
}

fn resolved_withdrawal_settings<'config>(
    config: &'config Config,
    scenario: &'config Scenario,
) -> Vec<WithdrawalSettingProjection<'config>> {
    resolved_event_occurrences(config, scenario)
        .into_iter()
        .filter_map(|occurrence| match occurrence.event {
            Event::SetMonthlyWithdrawal {
                name,
                asset_id,
                monthly_withdrawal,
                ..
            } => Some(WithdrawalSettingProjection {
                name,
                date: occurrence.date,
                asset_id,
                amount: &monthly_withdrawal.amount,
                currency: Some(&monthly_withdrawal.currency),
                is_adjustment: false,
                is_percentage: false,
            }),
            Event::AdjustMonthlyWithdrawal {
                name,
                asset_id,
                monthly_withdrawal,
                ..
            } => {
                let (amount, currency, is_percentage) = match monthly_withdrawal {
                    RecurringFlowAdjustment::Amount(adjustment) => {
                        (&adjustment.amount, Some(&adjustment.currency), false)
                    }
                    RecurringFlowAdjustment::Rate(adjustment) => (&adjustment.rate, None, true),
                };
                Some(WithdrawalSettingProjection {
                    name,
                    date: occurrence.date,
                    asset_id,
                    amount,
                    currency,
                    is_adjustment: true,
                    is_percentage,
                })
            }
            Event::AssetAdjustment { .. }
            | Event::SetMonthlyContribution { .. }
            | Event::AdjustMonthlyContribution { .. }
            | Event::SetAnnualExpectedReturn { .. } => None,
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
    resolved_event_occurrences(config, scenario)
        .into_iter()
        .filter_map(|occurrence| match occurrence.event {
            Event::SetAnnualExpectedReturn { asset_id, rate, .. } => Some(ResolvedReturnSetting {
                date: occurrence.date,
                asset_id,
                rate,
            }),
            Event::AssetAdjustment { .. }
            | Event::SetMonthlyContribution { .. }
            | Event::SetMonthlyWithdrawal { .. }
            | Event::AdjustMonthlyContribution { .. }
            | Event::AdjustMonthlyWithdrawal { .. } => None,
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
    withdrawal_settings: &[WithdrawalSettingProjection<'config>],
    return_settings: &[ResolvedReturnSetting<'config>],
) -> AssetProjection<'config> {
    let conversion_rate = config
        .conversion_rate_to_plan_currency(&asset.currency)
        .expect("validated asset currencies have a conversion rate");
    let mut month = config.plan.start;
    let opening_native_balance = asset
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
    let mut native_balance = Decimal::ZERO;
    let mut annual_expected_return = asset.annual_expected_return;
    let mut monthly_contribution = asset.monthly_contribution.clone();
    let mut monthly_withdrawal =
        asset
            .monthly_withdrawal
            .clone()
            .unwrap_or_else(|| MonthlyWithdrawal {
                amount: Decimal::ZERO,
                currency: asset.currency.clone(),
            });
    let month_count = config.plan.inclusive_month_count();
    let monthly_balances = (0..month_count)
        .map(|index| {
            let is_active = month >= asset.starts.expect("asset lifecycles resolve")
                && month <= asset.ends.expect("asset lifecycles resolve");
            if month == asset.starts.expect("asset lifecycles resolve") {
                native_balance = opening_native_balance;
            } else if !is_active {
                native_balance = Decimal::ZERO;
            }
            if let Some(setting) = return_settings
                .iter()
                .rev()
                .find(|setting| setting.date == month && setting.asset_id == asset.id)
            {
                annual_expected_return = *setting.rate;
            }
            for setting in contribution_settings
                .iter()
                .filter(|setting| setting.date == month && setting.asset_id == asset.id)
            {
                if setting.is_percentage {
                    monthly_contribution.amount *= Decimal::ONE + *setting.amount;
                } else if setting.is_adjustment {
                    monthly_contribution.amount += *setting.amount;
                } else {
                    monthly_contribution = MonthlyContribution {
                        amount: *setting.amount,
                        currency: setting
                            .currency
                            .expect("absolute contribution settings have a currency")
                            .clone(),
                    };
                }
            }
            for setting in withdrawal_settings
                .iter()
                .filter(|setting| setting.date == month && setting.asset_id == asset.id)
            {
                if setting.is_percentage {
                    monthly_withdrawal.amount *= Decimal::ONE + *setting.amount;
                } else if setting.is_adjustment {
                    monthly_withdrawal.amount += *setting.amount;
                } else {
                    monthly_withdrawal = MonthlyWithdrawal {
                        amount: *setting.amount,
                        currency: setting
                            .currency
                            .expect("absolute withdrawal settings have a currency")
                            .clone(),
                    };
                }
            }
            let adjustment_total = adjustments
                .iter()
                .filter(|adjustment| adjustment.date == month && adjustment.asset_id == asset.id)
                .map(|adjustment| *adjustment.amount)
                .sum::<Decimal>();
            let monthly_expected_return = monthly_rate(annual_expected_return);
            let native_passive_income = if is_active {
                native_balance * monthly_expected_return
            } else {
                Decimal::ZERO
            };
            let plan_passive_income = native_passive_income * conversion_rate;
            let native_monthly_contribution = if monthly_contribution.currency == asset.currency {
                monthly_contribution.amount
            } else {
                monthly_contribution.amount / conversion_rate
            };
            let native_monthly_withdrawal = if monthly_withdrawal.currency == asset.currency {
                monthly_withdrawal.amount
            } else {
                monthly_withdrawal.amount / conversion_rate
            };
            if is_active {
                native_balance = native_balance * (Decimal::ONE + monthly_expected_return)
                    + native_monthly_contribution
                    - native_monthly_withdrawal
                    + adjustment_total;
            }
            let pre_actual_native_balance = native_balance;
            if is_active {
                if let Some(actual_balance) = config
                    .actual_months
                    .get(&month)
                    .and_then(|actual| actual.balances.get(&asset.id))
                {
                    native_balance = actual_balance.0;
                }
            }
            let projection = AssetMonthProjection {
                month,
                is_active,
                annual_expected_return,
                monthly_contribution: monthly_contribution.clone(),
                monthly_withdrawal: monthly_withdrawal.clone(),
                native_balance,
                plan_balance: native_balance * conversion_rate,
                pre_actual_native_balance,
                pre_actual_plan_balance: pre_actual_native_balance * conversion_rate,
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
