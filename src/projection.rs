use rust_decimal::{Decimal, MathematicalOps};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::config::{
    AssetMilestone, Config, Currency, FutureLivingCost, Month, Plan, Scenario,
    TotalBalanceMilestone,
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
}

impl Serialize for ScenarioProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ScenarioProjection", 4)?;
        state.serialize_field("id", &self.scenario.id)?;
        state.serialize_field("name", &self.scenario.name)?;
        state.serialize_field("future_living_costs", &self.future_living_costs)?;
        state.serialize_field("asset_milestones", &self.asset_milestones)?;
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
    pub nominal_monthly_cost: PlanMoney<'config>,
}

impl<'config> FutureLivingCostProjection<'config> {
    pub fn id(&self) -> &'config str {
        &self.cost.id
    }

    pub fn name(&self) -> &'config str {
        &self.cost.name
    }
}

impl Serialize for FutureLivingCostProjection<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FutureLivingCostProjection", 3)?;
        state.serialize_field("id", &self.cost.id)?;
        state.serialize_field("name", &self.cost.name)?;
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

        Self {
            plan: PlanContext { plan: &config.plan },
            total_balance_milestones,
            scenarios: config
                .scenarios
                .iter()
                .map(|scenario| {
                    let inflation_factor =
                        (Decimal::ONE + scenario.annual_inflation).powd(inflation_years);
                    let costs = config
                        .future_living_costs
                        .iter()
                        .map(|cost| FutureLivingCostProjection {
                            cost,
                            nominal_monthly_cost: PlanMoney::calculated_in_plan_currency(
                                cost.monthly_cost * inflation_factor,
                            ),
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

                    ScenarioProjection {
                        scenario,
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
