use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    config::{Currency, Locale, Month},
    projection::{
        AssetMilestoneProjection, PlanMoney, PlanProjection, TotalBalanceMilestoneProjection,
    },
};

#[derive(Serialize)]
pub struct DashboardPresentation<'projection> {
    pub locale: &'static str,
    pub text: DashboardText,
    pub plan: DashboardPlanPresentation<'projection>,
    pub total_balance_milestones: Vec<DashboardMilestonePresentation<'projection>>,
    pub scenarios: Vec<DashboardScenarioPresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardText {
    pub scenario: &'static str,
    pub choose_scenario: &'static str,
    pub scenarios: &'static str,
    pub outcomes: &'static str,
    pub previous_outcome: &'static str,
    pub next_outcome: &'static str,
    pub future_living_costs: &'static str,
    pub monthly_total: &'static str,
    pub end_of_plan_nominal_money: &'static str,
    pub no_future_living_costs: &'static str,
    pub total_balance_target: &'static str,
    pub asset_balance_target: &'static str,
    pub projection_progress: &'static str,
    pub total_net_worth_canvas: &'static str,
    pub this_year: &'static str,
    pub mock_net_delta: &'static str,
    pub total_arc: &'static str,
    pub mock_net_worth_trajectory: &'static str,
    pub draft_canvas: &'static str,
    pub bold_ink_mocked_total: &'static str,
    pub dashed_milestone_target: &'static str,
    pub january: &'static str,
    pub april: &'static str,
    pub july: &'static str,
    pub october: &'static str,
    pub now: &'static str,
    pub goal: &'static str,
    pub one_million_horizon: &'static str,
}

#[derive(Serialize)]
pub struct DashboardPlanPresentation<'projection> {
    pub currency: &'projection Currency,
    pub start: Month,
    pub end: Month,
}

#[derive(Serialize)]
pub struct DashboardScenarioPresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub future_living_costs: DashboardFutureLivingCostsPresentation<'projection>,
    pub asset_milestones: Vec<DashboardMilestonePresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardFutureLivingCostsPresentation<'projection> {
    pub costs: Vec<DashboardFutureLivingCostPresentation<'projection>>,
    pub nominal_monthly_total: String,
}

#[derive(Serialize)]
pub struct DashboardFutureLivingCostPresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub nominal_monthly_cost: String,
}

#[derive(Serialize)]
pub struct DashboardMilestonePresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub target: String,
}

impl<'projection> DashboardPresentation<'projection> {
    pub fn new(projection: &'projection PlanProjection<'projection>, locale: Locale) -> Self {
        Self {
            locale: locale.html_language(),
            text: DashboardText::for_locale(locale),
            plan: DashboardPlanPresentation {
                currency: projection.plan.currency(),
                start: projection.plan.start(),
                end: projection.plan.end(),
            },
            total_balance_milestones: projection
                .total_balance_milestones
                .iter()
                .map(|milestone| DashboardMilestonePresentation::from_total(milestone, locale))
                .collect(),
            scenarios: projection
                .scenarios
                .iter()
                .map(|scenario| DashboardScenarioPresentation {
                    id: scenario.id(),
                    name: scenario.name(),
                    future_living_costs: DashboardFutureLivingCostsPresentation {
                        costs: scenario
                            .future_living_costs
                            .costs
                            .iter()
                            .map(|cost| DashboardFutureLivingCostPresentation {
                                id: cost.id(),
                                name: cost.name(),
                                nominal_monthly_cost: format_plan_money(
                                    &cost.nominal_monthly_cost,
                                    locale,
                                ),
                            })
                            .collect(),
                        nominal_monthly_total: format_plan_money(
                            &scenario.future_living_costs.nominal_monthly_total,
                            locale,
                        ),
                    },
                    asset_milestones: scenario
                        .asset_milestones
                        .iter()
                        .map(|milestone| {
                            DashboardMilestonePresentation::from_asset(milestone, locale)
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl<'projection> DashboardMilestonePresentation<'projection> {
    fn from_total(
        milestone: &TotalBalanceMilestoneProjection<'projection>,
        locale: Locale,
    ) -> Self {
        DashboardMilestonePresentation {
            id: milestone.id(),
            name: milestone.name(),
            target: format_plan_money(&milestone.target, locale),
        }
    }

    fn from_asset(milestone: &AssetMilestoneProjection<'projection>, locale: Locale) -> Self {
        DashboardMilestonePresentation {
            id: milestone.id(),
            name: milestone.name(),
            target: format_plan_money(&milestone.target, locale),
        }
    }
}

impl DashboardText {
    fn for_locale(locale: Locale) -> Self {
        match locale {
            Locale::EnUs => Self {
                scenario: "Scenario",
                choose_scenario: "Choose a scenario",
                scenarios: "scenarios",
                outcomes: "Outcomes",
                previous_outcome: "Previous outcome",
                next_outcome: "Next outcome",
                future_living_costs: "Future living costs",
                monthly_total: "Monthly total",
                end_of_plan_nominal_money: "End-of-plan nominal money",
                no_future_living_costs: "No future living costs configured.",
                total_balance_target: "Total-balance target",
                asset_balance_target: "Asset-balance target",
                projection_progress: "Progress will appear with projections.",
                total_net_worth_canvas: "Total net worth canvas",
                this_year: "this year",
                mock_net_delta: "mock net delta",
                total_arc: "Total arc",
                mock_net_worth_trajectory: "Mock net worth trajectory",
                draft_canvas: "Draft canvas",
                bold_ink_mocked_total: "Bold ink = mocked total",
                dashed_milestone_target: "Dashed = milestone target",
                january: "Jan",
                april: "Apr",
                july: "Jul",
                october: "Oct",
                now: "Now",
                goal: "Goal",
                one_million_horizon: "$1M horizon",
            },
            Locale::PtBr => Self {
                scenario: "Cenário",
                choose_scenario: "Escolha um cenário",
                scenarios: "cenários",
                outcomes: "Resultados",
                previous_outcome: "Resultado anterior",
                next_outcome: "Próximo resultado",
                future_living_costs: "Custos futuros de vida",
                monthly_total: "Total mensal",
                end_of_plan_nominal_money: "Valores nominais ao fim do plano",
                no_future_living_costs: "Nenhum custo futuro de vida configurado.",
                total_balance_target: "Meta de saldo total",
                asset_balance_target: "Meta de saldo do ativo",
                projection_progress: "O progresso aparecerá com as projeções.",
                total_net_worth_canvas: "Visão do patrimônio líquido",
                this_year: "este ano",
                mock_net_delta: "variação simulada do patrimônio",
                total_arc: "Trajetória total",
                mock_net_worth_trajectory: "Trajetória simulada do patrimônio",
                draft_canvas: "Rascunho",
                bold_ink_mocked_total: "Tinta forte = total simulado",
                dashed_milestone_target: "Tracejado = meta",
                january: "Jan",
                april: "Abr",
                july: "Jul",
                october: "Out",
                now: "Agora",
                goal: "Meta",
                one_million_horizon: "Meta de 1 mi",
            },
        }
    }
}

fn format_plan_money(money: &PlanMoney<'_>, locale: Locale) -> String {
    format_number(money.plan_amount(), locale)
}

fn format_number(amount: Decimal, locale: Locale) -> String {
    let formatted = format!("{:.2}", amount.round_dp(2));
    let (sign, absolute) = formatted
        .strip_prefix('-')
        .map_or(("", formatted.as_str()), |absolute| ("-", absolute));
    let (whole, fraction) = absolute
        .split_once('.')
        .expect("a Decimal formatted with two places always has a decimal point");
    let (group_separator, decimal_separator) = match locale {
        Locale::EnUs => (',', '.'),
        Locale::PtBr => ('.', ','),
    };
    let grouped_whole = whole
        .chars()
        .rev()
        .enumerate()
        .fold(String::new(), |mut result, (index, digit)| {
            if index > 0 && index % 3 == 0 {
                result.push(group_separator);
            }
            result.push(digit);
            result
        })
        .chars()
        .rev()
        .collect::<String>();

    format!("{sign}{grouped_whole}{decimal_separator}{fraction}")
}
