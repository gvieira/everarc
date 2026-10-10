use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    config::{Currency, Month},
    projection::{
        ActualBalanceComparisons, AppliedAssetEventKind, PlanProjection,
        ScenarioActualBalanceComparison, ScenarioProjection,
    },
};

/// A semantic, renderer-independent export of the calculated plan projection.
#[derive(Serialize)]
pub struct ProjectionExport<'config> {
    pub plan: PlanExport<'config>,
    pub total_balance_goals: Vec<TotalBalanceGoalExport<'config>>,
    pub scenarios: Vec<ScenarioExport<'config>>,
}

#[derive(Serialize)]
pub struct PlanExport<'config> {
    pub currency: &'config Currency,
    pub start: Month,
    pub end: Month,
    pub month_count: u32,
}

#[derive(Serialize)]
pub struct TotalBalanceGoalExport<'config> {
    pub id: &'config str,
    pub name: &'config str,
    pub target: String,
}

#[derive(Serialize)]
pub struct ScenarioExport<'config> {
    pub id: &'config str,
    pub name: &'config str,
    pub description: Option<&'config str>,
    pub selected: bool,
    pub outcome: ScenarioOutcomeExport,
    pub months: Vec<MonthExport<'config>>,
    pub events: Vec<EventExport<'config>>,
    pub future_living_costs: FutureLivingCostsExport<'config>,
    pub total_balance_goal_results: Vec<GoalResultExport<'config>>,
    pub asset_goals: Vec<AssetGoalExport<'config>>,
}

#[derive(Serialize)]
pub struct ScenarioOutcomeExport {
    pub end_balance: String,
    pub end_monthly_passive_income: String,
    pub estimated_end_monthly_withdrawal: String,
}

#[derive(Serialize)]
pub struct MonthExport<'config> {
    pub month: Month,
    pub actual_note: Option<&'config str>,
    pub total_balance: String,
    pub monthly_investment_rate: Option<String>,
    pub actual_total_balance: Option<ActualTotalExport>,
    pub assets: Vec<AssetMonthExport<'config>>,
}

#[derive(Serialize)]
pub struct ActualTotalExport {
    pub balance: String,
    pub planned_balance: String,
    pub difference_from_plan: String,
}

#[derive(Serialize)]
pub struct AssetMonthExport<'config> {
    pub id: &'config str,
    pub name: &'config str,
    pub currency: &'config Currency,
    pub active: bool,
    pub balance: String,
    pub plan_currency_balance: String,
    pub monthly_contribution: MoneyExport<'config>,
    pub monthly_withdrawal: MoneyExport<'config>,
    pub passive_income: String,
    pub plan_currency_passive_income: String,
    pub actual_balance: Option<ActualAssetExport>,
}

#[derive(Serialize)]
pub struct MoneyExport<'config> {
    pub amount: String,
    pub currency: &'config Currency,
}

#[derive(Serialize)]
pub struct ActualAssetExport {
    pub balance: String,
    pub planned_balance: String,
    pub difference_from_plan: String,
    pub plan_currency_balance: String,
    pub planned_plan_currency_balance: String,
    pub plan_currency_difference_from_plan: String,
}

#[derive(Serialize)]
pub struct EventExport<'config> {
    pub name: &'config str,
    pub month: Month,
    pub asset_id: &'config str,
    pub kind: AppliedAssetEventKind,
    pub amount: String,
    pub currency: Option<&'config Currency>,
}

#[derive(Serialize)]
pub struct FutureLivingCostsExport<'config> {
    pub end_monthly_total: String,
    pub costs: Vec<FutureLivingCostExport<'config>>,
}

#[derive(Serialize)]
pub struct FutureLivingCostExport<'config> {
    pub id: &'config str,
    pub name: &'config str,
    pub description: Option<&'config str>,
    pub annual_inflation: String,
    pub today_monthly_cost: String,
    pub end_monthly_cost: String,
}

#[derive(Serialize)]
pub struct GoalResultExport<'config> {
    pub id: &'config str,
    pub reached_month: Option<Month>,
}

#[derive(Serialize)]
pub struct AssetGoalExport<'config> {
    pub id: &'config str,
    pub name: &'config str,
    pub asset_id: &'config str,
    pub target: String,
    pub reached_month: Option<Month>,
}

impl<'config> ProjectionExport<'config> {
    pub fn with_actual_balances(
        config: &'config crate::config::Config,
        projection: &'config PlanProjection<'config>,
    ) -> Self {
        let comparisons = ActualBalanceComparisons::new(config, projection);
        Self::from_comparisons(projection, &comparisons)
    }

    fn from_comparisons(
        projection: &'config PlanProjection<'config>,
        comparisons: &ActualBalanceComparisons<'config>,
    ) -> Self {
        Self {
            plan: PlanExport {
                currency: projection.plan.currency(),
                start: projection.plan.start(),
                end: projection.plan.end(),
                month_count: projection.plan.inclusive_month_count(),
            },
            total_balance_goals: projection
                .total_balance_milestones
                .iter()
                .map(|goal| TotalBalanceGoalExport {
                    id: goal.id(),
                    name: goal.name(),
                    target: decimal(goal.target.plan_amount()),
                })
                .collect(),
            scenarios: projection
                .scenarios
                .iter()
                .map(|scenario| {
                    let comparison = comparisons
                        .scenarios
                        .iter()
                        .find(|comparison| comparison.scenario_id == scenario.id());
                    scenario_export(projection, scenario, comparison)
                })
                .collect(),
        }
    }
}

fn scenario_export<'config>(
    projection: &'config PlanProjection<'config>,
    scenario: &'config ScenarioProjection<'config>,
    comparison: Option<&ScenarioActualBalanceComparison<'config>>,
) -> ScenarioExport<'config> {
    let final_month = scenario
        .total_net_worth
        .last()
        .expect("projections include every plan month");
    let end_monthly_passive_income = scenario
        .assets
        .iter()
        .filter_map(|asset| asset.monthly_balances.last())
        .map(|month| month.plan_passive_income)
        .sum();

    ScenarioExport {
        id: scenario.id(),
        name: scenario.name(),
        description: scenario.description(),
        selected: scenario.is_selected(),
        outcome: ScenarioOutcomeExport {
            end_balance: decimal(final_month.balance),
            end_monthly_passive_income: decimal(end_monthly_passive_income),
            estimated_end_monthly_withdrawal: decimal(
                final_month.balance * projection.plan.withdrawal_rate() / Decimal::from(12),
            ),
        },
        months: scenario
            .total_net_worth
            .iter()
            .enumerate()
            .map(|(index, total)| MonthExport {
                month: total.month,
                actual_note: comparison
                    .and_then(|comparison| comparison.months.get(index))
                    .and_then(|month| month.note),
                total_balance: decimal(total.balance),
                monthly_investment_rate: total.monthly_investment_rate.map(decimal),
                actual_total_balance: comparison
                    .and_then(|comparison| comparison.months.get(index))
                    .and_then(|month| month.total.as_ref())
                    .map(|total| ActualTotalExport {
                        balance: decimal(total.actual_balance),
                        planned_balance: decimal(total.planned_balance),
                        difference_from_plan: decimal(total.difference),
                    }),
                assets: scenario
                    .assets
                    .iter()
                    .map(|asset| {
                        let month = &asset.monthly_balances[index];
                        let actual = comparison
                            .and_then(|comparison| comparison.months.get(index))
                            .and_then(|month| {
                                month
                                    .assets
                                    .iter()
                                    .find(|actual| actual.asset_id == asset.id())
                            });
                        AssetMonthExport {
                            id: asset.id(),
                            name: asset.name(),
                            currency: asset.currency(),
                            active: month.is_active,
                            balance: decimal(month.native_balance),
                            plan_currency_balance: decimal(month.plan_balance),
                            monthly_contribution: MoneyExport {
                                amount: decimal(month.monthly_contribution.amount),
                                currency: &month.monthly_contribution.currency,
                            },
                            monthly_withdrawal: MoneyExport {
                                amount: decimal(month.monthly_withdrawal.amount),
                                currency: &month.monthly_withdrawal.currency,
                            },
                            passive_income: decimal(month.native_passive_income),
                            plan_currency_passive_income: decimal(month.plan_passive_income),
                            actual_balance: actual.map(|actual| ActualAssetExport {
                                balance: decimal(actual.actual_native_balance),
                                planned_balance: decimal(actual.planned_native_balance),
                                difference_from_plan: decimal(actual.native_difference),
                                plan_currency_balance: decimal(actual.actual_plan_balance),
                                planned_plan_currency_balance: decimal(actual.planned_plan_balance),
                                plan_currency_difference_from_plan: decimal(actual.plan_difference),
                            }),
                        }
                    })
                    .collect(),
            })
            .collect(),
        events: scenario
            .asset_events
            .iter()
            .map(|event| EventExport {
                name: event.name,
                month: event.date,
                asset_id: event.asset_id,
                kind: event.kind,
                amount: decimal(*event.amount),
                currency: event.currency,
            })
            .collect(),
        future_living_costs: FutureLivingCostsExport {
            end_monthly_total: decimal(
                scenario
                    .future_living_costs
                    .nominal_monthly_total
                    .plan_amount(),
            ),
            costs: scenario
                .future_living_costs
                .costs
                .iter()
                .map(|cost| FutureLivingCostExport {
                    id: cost.id(),
                    name: cost.name(),
                    description: cost.description(),
                    annual_inflation: decimal(cost.annual_inflation),
                    today_monthly_cost: decimal(cost.today_money_monthly_cost()),
                    end_monthly_cost: decimal(cost.nominal_monthly_cost.plan_amount()),
                })
                .collect(),
        },
        total_balance_goal_results: projection
            .total_balance_milestones
            .iter()
            .map(|goal| GoalResultExport {
                id: goal.id(),
                reached_month: scenario
                    .total_net_worth
                    .iter()
                    .find(|month| month.balance >= goal.target.plan_amount())
                    .map(|month| month.month),
            })
            .collect(),
        asset_goals: scenario
            .asset_milestones
            .iter()
            .map(|goal| AssetGoalExport {
                id: goal.id(),
                name: goal.name(),
                asset_id: goal.asset_id(),
                target: decimal(goal.target.plan_amount()),
                reached_month: scenario
                    .assets
                    .iter()
                    .find(|asset| asset.id() == goal.asset_id())
                    .and_then(|asset| {
                        asset
                            .monthly_balances
                            .iter()
                            .find(|month| month.plan_balance >= goal.target.plan_amount())
                            .map(|month| month.month)
                    }),
            })
            .collect(),
    }
}

fn decimal(value: Decimal) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn exports_pre_checkpoint_forecasts_and_rebased_future_balances() {
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
rate = "2"

[actual_months."2026-01"]
note = "Unexpected expense & extra savings."
balances = { cash = "95", fund = "20" }

[actual_months."2026-02"]
note = "Balances not recorded yet."

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
id = "opening"
name = "Opening"
currency = "USD"
value = "100"

[[scenarios.assets]]
id = "fund"
name = "Fund"
currency = "EUR"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "EUR" }

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening"
currency = "EUR"
value = "10"

[[scenarios]]
id = "child"
name = "Child"
extends = "base"
"#,
        )
        .expect("configuration parses");
        config.validate().expect("configuration validates");
        let projection = PlanProjection::from(&config);
        let json =
            serde_json::to_value(ProjectionExport::with_actual_balances(&config, &projection))
                .expect("export serializes");
        let january = &json["scenarios"][0]["months"][0];
        let cash = &january["assets"][0];
        let fund = &january["assets"][1];

        assert_eq!(
            january["actual_note"],
            "Unexpected expense & extra savings."
        );
        assert_eq!(january["total_balance"], "135");
        assert_eq!(january["actual_total_balance"]["balance"], "135");
        assert_eq!(january["actual_total_balance"]["planned_balance"], "130");
        assert_eq!(january["actual_total_balance"]["difference_from_plan"], "5");
        assert_eq!(cash["balance"], "95");
        assert_eq!(cash["actual_balance"]["planned_balance"], "110");
        assert_eq!(cash["actual_balance"]["balance"], "95");
        assert_eq!(cash["actual_balance"]["difference_from_plan"], "-15");
        assert_eq!(fund["actual_balance"]["planned_balance"], "10");
        assert_eq!(
            fund["actual_balance"]["planned_plan_currency_balance"],
            "20"
        );
        assert_eq!(fund["actual_balance"]["plan_currency_balance"], "40");
        assert_eq!(
            fund["actual_balance"]["plan_currency_difference_from_plan"],
            "20"
        );

        let february = &json["scenarios"][0]["months"][1];
        assert_eq!(february["actual_note"], "Balances not recorded yet.");
        assert_eq!(february["total_balance"], "145");
        assert_eq!(february["assets"][0]["balance"], "105");
        assert!(february["actual_total_balance"].is_null());
        assert!(february["assets"][0]["actual_balance"].is_null());
        assert!(json["scenarios"][0]["months"][2]["actual_note"].is_null());
        for index in 0..3 {
            assert_eq!(
                json["scenarios"][0]["months"][index]["actual_note"],
                json["scenarios"][1]["months"][index]["actual_note"]
            );
        }
    }

    #[test]
    fn exports_only_enabled_event_occurrences_and_their_calculated_balances() {
        let mut config: Config =
            toml::from_str(include_str!("../tests/fixtures/event-overrides.toml")).unwrap();
        config.validate().unwrap();
        let projection = PlanProjection::from(&config);
        let json =
            serde_json::to_value(ProjectionExport::with_actual_balances(&config, &projection))
                .unwrap();
        let scenarios = json["scenarios"].as_array().unwrap();
        let scenario = |id: &str| {
            scenarios
                .iter()
                .find(|scenario| scenario["id"] == id)
                .unwrap()
        };

        for id in ["disabled", "descendant"] {
            let scenario = scenario(id);
            assert!(scenario["events"].as_array().unwrap().is_empty());
            assert_eq!(scenario["outcome"]["end_balance"], "180");
            for (month, balance) in scenario["months"]
                .as_array()
                .unwrap()
                .iter()
                .zip(["60", "120", "180"])
            {
                assert_eq!(month["total_balance"], balance);
                assert_eq!(month["assets"][0]["monthly_contribution"]["amount"], "100");
                assert_eq!(month["assets"][0]["monthly_withdrawal"]["amount"], "40");
            }
        }

        let revived = scenario("revived");
        let events = revived["events"].as_array().unwrap();
        assert_eq!(events.len(), 3);
        for (event, month) in events.iter().zip(["2026-01", "2026-02", "2026-03"]) {
            assert_eq!(event["name"], "Deposit");
            assert_eq!(event["month"], month);
            assert_eq!(event["asset_id"], "cash");
            assert_eq!(event["kind"], "adjustment");
            assert_eq!(event["amount"], "75");
        }
        assert_eq!(revived["outcome"]["end_balance"], "405");
        for (month, balance) in revived["months"]
            .as_array()
            .unwrap()
            .iter()
            .zip(["135", "270", "405"])
        {
            assert_eq!(month["total_balance"], balance);
        }

        let parent_events = scenario("base")["events"].as_array().unwrap();
        assert_eq!(parent_events.len(), 12);
        let deposits = parent_events
            .iter()
            .filter(|event| event["name"] == "Deposit")
            .collect::<Vec<_>>();
        assert_eq!(deposits.len(), 3);
        assert!(deposits.iter().all(|event| event["amount"] == "50"));
    }
}
