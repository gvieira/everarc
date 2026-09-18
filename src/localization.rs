use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    config::{Currency, Locale, Month},
    projection::{AppliedAssetEventKind, PlanMoney, PlanProjection},
};

const CHART_LEFT: u32 = 80;
const CHART_RIGHT: u32 = 950;
const CHART_TOP: u32 = 75;
const CHART_BOTTOM: u32 = 435;

#[derive(Serialize)]
pub struct DashboardPresentation<'projection> {
    pub locale: &'static str,
    pub text: DashboardText,
    pub plan: DashboardPlanPresentation<'projection>,
    pub chart: DashboardChartPresentation<'projection>,
    pub scenarios: Vec<DashboardScenarioPresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardText {
    pub scenario: &'static str,
    pub choose_scenario: &'static str,
    pub selected: &'static str,
    pub scenarios: &'static str,
    pub annual_inflation: &'static str,
    pub outcomes: &'static str,
    pub previous_outcome: &'static str,
    pub next_outcome: &'static str,
    pub future_living_costs: &'static str,
    pub inflation_adjusted: &'static str,
    pub monthly_total: &'static str,
    pub end_of_plan_nominal_money: &'static str,
    pub no_future_living_costs: &'static str,
    pub total_balance_target: &'static str,
    pub asset_balance_target: &'static str,
    pub reached_in: &'static str,
    pub not_reached_by: &'static str,
    pub total_net_worth: &'static str,
    pub plan_summary: &'static str,
    pub end_of_plan_passive_income: &'static str,
    pub conversion_rates: &'static str,
    pub no_conversion_rates: &'static str,
    pub projection: &'static str,
    pub month: &'static str,
    pub total: &'static str,
    pub passive_income: &'static str,
    pub assets: &'static str,
    pub annual_return: &'static str,
    pub monthly_contribution: &'static str,
    pub per_month: &'static str,
    pub total_net_worth_legend: &'static str,
    pub goals_legend: &'static str,
    pub zoom_in: &'static str,
    pub zoom_out: &'static str,
    pub reset_zoom: &'static str,
}

#[derive(Serialize)]
pub struct DashboardPlanPresentation<'projection> {
    pub currency: &'projection Currency,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub conversion_rates: Vec<DashboardConversionRatePresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardConversionRatePresentation<'projection> {
    pub from: &'projection Currency,
    pub to: &'projection Currency,
    pub rate: String,
}

#[derive(Serialize)]
pub struct DashboardChartPresentation<'projection> {
    pub y_ticks: Vec<DashboardChartTick>,
    pub x_grid: Vec<String>,
    pub x_labels: Vec<DashboardChartLabel>,
    pub milestones: Vec<DashboardChartMilestone<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartTick {
    pub y: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct DashboardChartLabel {
    pub x: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct DashboardChartMilestone<'projection> {
    pub name: &'projection str,
    pub line_y: String,
    pub label_y: String,
}

#[derive(Serialize)]
pub struct DashboardScenarioPresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub description: Option<&'projection str>,
    pub selected: bool,
    pub is_default: bool,
    pub annual_inflation: String,
    pub end_of_plan_passive_income: String,
    pub chart_path: String,
    pub chart_assets: Vec<DashboardChartAssetLinePresentation<'projection>>,
    pub chart_months: Vec<DashboardChartMonthPresentation<'projection>>,
    pub future_living_costs: DashboardFutureLivingCostsPresentation<'projection>,
    pub total_balance_milestones: Vec<DashboardMilestonePresentation<'projection>>,
    pub asset_milestones: Vec<DashboardMilestonePresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartAssetLinePresentation<'projection> {
    pub name: &'projection str,
    pub path: String,
    pub color_index: usize,
    pub event_markers: Vec<DashboardChartEventMarker>,
    pub milestone_markers: Vec<DashboardChartAssetMilestone<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartEventMarker {
    pub x: String,
    pub y: String,
}

#[derive(Serialize)]
pub struct DashboardChartAssetMilestone<'projection> {
    pub name: &'projection str,
    pub target: String,
    pub x: String,
    pub y: String,
}

#[derive(Serialize)]
pub struct DashboardChartMonthPresentation<'projection> {
    pub month: String,
    pub x: String,
    pub y: String,
    pub balance: String,
    pub total: String,
    pub passive_income: String,
    pub assets: Vec<DashboardChartAssetPresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartAssetPresentation<'projection> {
    pub name: &'projection str,
    pub native_balance: String,
    pub annual_expected_return: String,
    pub monthly_contribution: String,
    pub monthly_contribution_currency: &'projection Currency,
    pub passive_income: String,
    pub is_plan_currency: bool,
    pub y: String,
    pub balance: String,
    pub color_index: usize,
    pub currency: &'projection Currency,
    pub comparable_plan_balance: Option<String>,
    pub events: Vec<DashboardAssetEventPresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardAssetEventPresentation<'projection> {
    pub name: &'projection str,
    pub value: String,
}

#[derive(Serialize)]
pub struct DashboardFutureLivingCostsPresentation<'projection> {
    pub costs: Vec<DashboardFutureLivingCostPresentation<'projection>>,
    pub nominal_monthly_total: String,
    pub today_money_monthly_total: String,
}

#[derive(Serialize)]
pub struct DashboardFutureLivingCostPresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub tooltip: Option<String>,
    pub nominal_share: String,
    pub today_money_share: String,
    pub nominal_monthly_cost: String,
    pub today_money_monthly_cost: String,
}

#[derive(Serialize)]
pub struct DashboardMilestonePresentation<'projection> {
    pub id: &'projection str,
    pub name: &'projection str,
    pub target: String,
    pub timing: String,
}

impl<'projection> DashboardPresentation<'projection> {
    pub fn new(projection: &'projection PlanProjection<'projection>, locale: Locale) -> Self {
        let default_scenario_id = projection
            .scenarios
            .iter()
            .find(|scenario| scenario.is_selected())
            .or_else(|| projection.scenarios.first())
            .map(|scenario| scenario.id());
        let chart_maximum = projection
            .scenarios
            .iter()
            .flat_map(|scenario| scenario.total_net_worth.iter().map(|month| month.balance))
            .chain(
                projection
                    .total_balance_milestones
                    .iter()
                    .map(|milestone| milestone.target.plan_amount()),
            )
            .chain(projection.scenarios.iter().flat_map(|scenario| {
                scenario
                    .asset_milestones
                    .iter()
                    .map(|milestone| milestone.target.plan_amount())
            }))
            .max()
            .filter(|maximum| *maximum > Decimal::ZERO)
            .unwrap_or(Decimal::ONE);
        let chart_scale = nice_chart_scale(chart_maximum);
        let text = DashboardText::for_locale(locale);
        let chart = DashboardChartPresentation {
            y_ticks: chart_y_ticks(chart_scale, locale),
            x_grid: projection
                .scenarios
                .first()
                .map(|scenario| chart_x_grid(&scenario.total_net_worth))
                .unwrap_or_default(),
            x_labels: projection
                .scenarios
                .first()
                .map(|scenario| chart_x_labels(&scenario.total_net_worth, locale))
                .unwrap_or_default(),
            milestones: projection
                .total_balance_milestones
                .iter()
                .map(|milestone| DashboardChartMilestone {
                    name: milestone.name(),
                    line_y: chart_y(milestone.target.plan_amount(), chart_scale.maximum),
                    label_y: chart_milestone_label_y(
                        milestone.target.plan_amount(),
                        chart_scale.maximum,
                    ),
                })
                .collect(),
        };

        Self {
            locale: locale.html_language(),
            plan: DashboardPlanPresentation {
                currency: projection.plan.currency(),
                start: format_month(projection.plan.start(), locale),
                end: format_month(projection.plan.end(), locale),
                duration: format_duration(projection.plan.inclusive_month_count(), locale),
                conversion_rates: projection
                    .plan
                    .conversion_rates()
                    .iter()
                    .map(|conversion_rate| DashboardConversionRatePresentation {
                        from: &conversion_rate.from,
                        to: &conversion_rate.to,
                        rate: format_number(conversion_rate.rate, locale),
                    })
                    .collect(),
            },
            chart,
            scenarios: projection
                .scenarios
                .iter()
                .map(|scenario| DashboardScenarioPresentation {
                    id: scenario.id(),
                    name: scenario.name(),
                    description: scenario.description(),
                    selected: scenario.is_selected(),
                    is_default: Some(scenario.id()) == default_scenario_id,
                    annual_inflation: format_percentage(scenario.annual_inflation(), locale),
                    end_of_plan_passive_income: format_number(
                        scenario
                            .assets
                            .iter()
                            .filter_map(|asset| asset.monthly_balances.last())
                            .map(|month| month.plan_passive_income)
                            .sum(),
                        locale,
                    ),
                    chart_path: chart_path(&scenario.total_net_worth, chart_scale.maximum),
                    chart_assets: scenario
                        .assets
                        .iter()
                        .enumerate()
                        .map(|(color_index, asset)| DashboardChartAssetLinePresentation {
                            name: asset.name(),
                            path: asset_chart_path(asset, chart_scale.maximum),
                            color_index,
                            event_markers: asset
                                .monthly_balances
                                .iter()
                                .enumerate()
                                .filter(|(_, balance)| {
                                    scenario.asset_events.iter().any(|event| {
                                        event.date == balance.month && event.asset_id == asset.id()
                                    })
                                })
                                .map(|(index, balance)| DashboardChartEventMarker {
                                    x: chart_x(index, asset.monthly_balances.len() - 1),
                                    y: chart_y(balance.plan_balance, chart_scale.maximum),
                                })
                                .collect(),
                            milestone_markers: scenario
                                .asset_milestones
                                .iter()
                                .filter(|milestone| milestone.asset_id() == asset.id())
                                .filter_map(|milestone| {
                                    asset
                                        .monthly_balances
                                        .iter()
                                        .enumerate()
                                        .find(|(_, balance)| {
                                            balance.plan_balance >= milestone.target.plan_amount()
                                        })
                                        .map(|(index, balance)| DashboardChartAssetMilestone {
                                            name: milestone.name(),
                                            target: format_plan_money(&milestone.target, locale),
                                            x: chart_x(index, asset.monthly_balances.len() - 1),
                                            y: chart_y(balance.plan_balance, chart_scale.maximum),
                                        })
                                })
                                .collect(),
                        })
                        .collect(),
                    chart_months: chart_months(
                        scenario,
                        chart_scale.maximum,
                        locale,
                        projection.plan.currency(),
                        &text,
                    ),
                    future_living_costs: DashboardFutureLivingCostsPresentation {
                        costs: scenario
                            .future_living_costs
                            .costs
                            .iter()
                            .map(|cost| DashboardFutureLivingCostPresentation {
                                id: cost.id(),
                                name: cost.name(),
                                tooltip: Some(match cost.description() {
                                    Some(description) => format!(
                                        "{description}\n{}: {}",
                                        text.annual_inflation,
                                        format_percentage(cost.annual_inflation, locale),
                                    ),
                                    None => format!(
                                        "{}: {}",
                                        text.annual_inflation,
                                        format_percentage(cost.annual_inflation, locale),
                                    ),
                                }),
                                nominal_share: format_rounded_percentage(
                                    cost.nominal_monthly_cost.plan_amount()
                                        / scenario
                                            .future_living_costs
                                            .nominal_monthly_total
                                            .plan_amount(),
                                    locale,
                                ),
                                today_money_share: format_rounded_percentage(
                                    cost.today_money_monthly_cost()
                                        / scenario
                                            .future_living_costs
                                            .costs
                                            .iter()
                                            .map(|cost| cost.today_money_monthly_cost())
                                            .sum::<Decimal>(),
                                    locale,
                                ),
                                nominal_monthly_cost: format_plan_money(
                                    &cost.nominal_monthly_cost,
                                    locale,
                                ),
                                today_money_monthly_cost: format_number(
                                    cost.today_money_monthly_cost(),
                                    locale,
                                ),
                            })
                            .collect(),
                        nominal_monthly_total: format_plan_money(
                            &scenario.future_living_costs.nominal_monthly_total,
                            locale,
                        ),
                        today_money_monthly_total: format_number(
                            scenario
                                .future_living_costs
                                .costs
                                .iter()
                                .map(|cost| cost.today_money_monthly_cost())
                                .sum(),
                            locale,
                        ),
                    },
                    total_balance_milestones: projection
                        .total_balance_milestones
                        .iter()
                        .map(|milestone| DashboardMilestonePresentation {
                            id: milestone.id(),
                            name: milestone.name(),
                            target: format_plan_money(&milestone.target, locale),
                            timing: milestone_timing(
                                scenario
                                    .total_net_worth
                                    .iter()
                                    .find(|month| month.balance >= milestone.target.plan_amount())
                                    .map(|month| month.month),
                                projection.plan.end(),
                                &text,
                                locale,
                            ),
                        })
                        .collect(),
                    asset_milestones: scenario
                        .asset_milestones
                        .iter()
                        .map(|milestone| DashboardMilestonePresentation {
                            id: milestone.id(),
                            name: milestone.name(),
                            target: format_plan_money(&milestone.target, locale),
                            timing: milestone_timing(
                                scenario
                                    .assets
                                    .iter()
                                    .find(|asset| asset.id() == milestone.asset_id())
                                    .and_then(|asset| {
                                        asset.monthly_balances.iter().find(|month| {
                                            month.plan_balance >= milestone.target.plan_amount()
                                        })
                                    })
                                    .map(|month| month.month),
                                projection.plan.end(),
                                &text,
                                locale,
                            ),
                        })
                        .collect(),
                })
                .collect(),
            text,
        }
    }
}

fn milestone_timing(
    reached: Option<Month>,
    plan_end: Month,
    text: &DashboardText,
    locale: Locale,
) -> String {
    match reached {
        Some(month) => format!("{} {}", text.reached_in, format_month(month, locale)),
        None => format!("{} {}", text.not_reached_by, format_month(plan_end, locale)),
    }
}

fn format_month(month: Month, _locale: Locale) -> String {
    format!("{:02}/{}", month.month(), month.year())
}

impl DashboardText {
    fn for_locale(locale: Locale) -> Self {
        match locale {
            Locale::EnUs => Self {
                scenario: "Scenario",
                choose_scenario: "Choose a scenario",
                selected: "Selected",
                scenarios: "scenarios",
                annual_inflation: "Annual inflation",
                outcomes: "Outcomes",
                previous_outcome: "Previous outcome",
                next_outcome: "Next outcome",
                future_living_costs: "Future living costs",
                inflation_adjusted: "Inflation-adjusted",
                monthly_total: "Monthly total",
                end_of_plan_nominal_money: "End-of-plan nominal money",
                no_future_living_costs: "No future living costs configured.",
                total_balance_target: "Total-balance target",
                asset_balance_target: "Asset-balance target",
                reached_in: "Reached in",
                not_reached_by: "Not reached by",
                total_net_worth: "Total net worth",
                plan_summary: "Plan summary",
                end_of_plan_passive_income: "Monthly passive income at the end of the plan",
                conversion_rates: "Conversion rates",
                no_conversion_rates: "No conversion rates configured.",
                projection: "Projection",
                month: "Month",
                total: "Total",
                passive_income: "Passive income",
                assets: "Assets",
                annual_return: "Annual return",
                monthly_contribution: "Monthly contribution",
                per_month: "/month",
                total_net_worth_legend: "Total net worth",
                goals_legend: "Goals",
                zoom_in: "Zoom in",
                zoom_out: "Zoom out",
                reset_zoom: "Show full plan",
            },
            Locale::PtBr => Self {
                scenario: "Cenário",
                choose_scenario: "Escolha um cenário",
                selected: "Selecionado",
                scenarios: "cenários",
                annual_inflation: "Inflação anual",
                outcomes: "Resultados",
                previous_outcome: "Resultado anterior",
                next_outcome: "Próximo resultado",
                future_living_costs: "Custos futuros de vida",
                inflation_adjusted: "Ajustado pela inflação",
                monthly_total: "Total mensal",
                end_of_plan_nominal_money: "Valores nominais ao fim do plano",
                no_future_living_costs: "Nenhum custo futuro de vida configurado.",
                total_balance_target: "Meta de saldo total",
                asset_balance_target: "Meta de saldo do ativo",
                reached_in: "Atingida em",
                not_reached_by: "Não atingida até",
                total_net_worth: "Patrimônio líquido total",
                plan_summary: "Resumo do plano",
                end_of_plan_passive_income: "Renda passiva mensal ao fim do plano",
                conversion_rates: "Taxas de conversão",
                no_conversion_rates: "Nenhuma taxa de conversão configurada.",
                projection: "Projeção",
                month: "Mês",
                total: "Total",
                passive_income: "Renda passiva",
                assets: "Ativos",
                annual_return: "Retorno anual",
                monthly_contribution: "Contribuição mensal",
                per_month: "/mês",
                total_net_worth_legend: "Patrimônio líquido total",
                goals_legend: "Metas",
                zoom_in: "Aumentar zoom",
                zoom_out: "Diminuir zoom",
                reset_zoom: "Mostrar plano completo",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ChartScale {
    maximum: Decimal,
    step: Decimal,
}

fn nice_chart_scale(maximum: Decimal) -> ChartScale {
    let desired_step = maximum / Decimal::from(4);
    let mut magnitude = Decimal::ONE;
    while desired_step >= magnitude * Decimal::from(10) {
        magnitude *= Decimal::from(10);
    }
    while desired_step < magnitude {
        magnitude /= Decimal::from(10);
    }

    let normalized_step = desired_step / magnitude;
    let factor = if normalized_step <= Decimal::new(15, 1) {
        Decimal::ONE
    } else if normalized_step <= Decimal::new(225, 2) {
        Decimal::from(2)
    } else if normalized_step <= Decimal::new(35, 1) {
        Decimal::new(25, 1)
    } else if normalized_step <= Decimal::new(75, 1) {
        Decimal::from(5)
    } else {
        Decimal::from(10)
    };
    let step = factor * magnitude;
    let rounded_maximum = (maximum / step).ceil() * step;

    ChartScale {
        maximum: rounded_maximum,
        step,
    }
}

fn chart_y_ticks(scale: ChartScale, locale: Locale) -> Vec<DashboardChartTick> {
    std::iter::successors(Some(Decimal::ZERO), |value| {
        let next = *value + scale.step;
        (next <= scale.maximum).then_some(next)
    })
    .map(|value| DashboardChartTick {
        y: chart_y(value, scale.maximum),
        label: format_chart_axis_label(value, locale),
    })
    .collect()
}

fn chart_x_labels(
    months: &[crate::projection::TotalNetWorthMonthProjection],
    locale: Locale,
) -> Vec<DashboardChartLabel> {
    let last_index = months.len().saturating_sub(1);
    [
        0,
        last_index / 4,
        last_index / 2,
        last_index * 3 / 4,
        last_index,
    ]
    .into_iter()
    .fold(Vec::new(), |mut labels, index| {
        if labels
            .iter()
            .any(|label: &DashboardChartLabel| label.x == chart_x(index, last_index))
        {
            return labels;
        }
        let Some(month) = months.get(index) else {
            return labels;
        };
        labels.push(DashboardChartLabel {
            x: chart_x(index, last_index),
            label: format_month(month.month, locale),
        });
        labels
    })
}

fn chart_x_grid(months: &[crate::projection::TotalNetWorthMonthProjection]) -> Vec<String> {
    let last_index = months.len().saturating_sub(1);
    months
        .iter()
        .enumerate()
        .filter(|(index, projection)| {
            *index == 0
                || *index == last_index
                || (projection.month.month() == 1 && projection.month.year().is_multiple_of(5))
        })
        .map(|(index, _)| chart_x(index, last_index))
        .collect()
}

fn chart_path(
    months: &[crate::projection::TotalNetWorthMonthProjection],
    maximum: Decimal,
) -> String {
    let last_index = months.len().saturating_sub(1);
    months
        .iter()
        .enumerate()
        .map(|(index, month)| {
            let command = if index == 0 { "M" } else { "L" };
            format!(
                "{command} {} {}",
                chart_x(index, last_index),
                chart_y(month.balance, maximum)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn asset_chart_path(asset: &crate::projection::AssetProjection<'_>, maximum: Decimal) -> String {
    let last_index = asset.monthly_balances.len().saturating_sub(1);
    asset
        .monthly_balances
        .iter()
        .enumerate()
        .map(|(index, month)| {
            let command = if index == 0 { "M" } else { "L" };
            format!(
                "{command} {} {}",
                chart_x(index, last_index),
                chart_y(month.plan_balance, maximum)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn chart_months<'projection>(
    scenario: &'projection crate::projection::ScenarioProjection<'projection>,
    maximum: Decimal,
    locale: Locale,
    plan_currency: &'projection Currency,
    text: &DashboardText,
) -> Vec<DashboardChartMonthPresentation<'projection>> {
    let last_index = scenario.total_net_worth.len().saturating_sub(1);
    scenario
        .total_net_worth
        .iter()
        .enumerate()
        .map(|(index, total)| DashboardChartMonthPresentation {
            month: format_month(total.month, locale),
            x: chart_x(index, last_index),
            y: chart_y(total.balance, maximum),
            balance: total.balance.to_string(),
            total: format_number(total.balance, locale),
            passive_income: format_number(
                scenario
                    .assets
                    .iter()
                    .map(|asset| asset.monthly_balances[index].plan_passive_income)
                    .sum(),
                locale,
            ),
            assets: scenario
                .assets
                .iter()
                .enumerate()
                .map(|(asset_index, asset)| {
                    let balance = &asset.monthly_balances[index];
                    let comparable_plan_balance = (asset.currency() != plan_currency)
                        .then(|| format_number(balance.plan_balance, locale));
                    DashboardChartAssetPresentation {
                        name: asset.name(),
                        native_balance: format_number(balance.native_balance, locale),
                        annual_expected_return: format_percentage(
                            balance.annual_expected_return,
                            locale,
                        ),
                        monthly_contribution: format_number(
                            balance.monthly_contribution.amount,
                            locale,
                        ),
                        monthly_contribution_currency: &balance.monthly_contribution.currency,
                        passive_income: format_number(balance.plan_passive_income, locale),
                        is_plan_currency: asset.currency() == plan_currency,
                        y: chart_y(balance.plan_balance, maximum),
                        balance: balance.plan_balance.to_string(),
                        color_index: asset_index,
                        currency: asset.currency(),
                        comparable_plan_balance,
                        events: scenario
                            .asset_events
                            .iter()
                            .filter(|event| {
                                event.date == total.month && event.asset_id == asset.id()
                            })
                            .map(|event| DashboardAssetEventPresentation {
                                name: event.name,
                                value: match event.kind {
                                    AppliedAssetEventKind::Adjustment => {
                                        format_signed_number(*event.amount, locale)
                                    }
                                    AppliedAssetEventKind::ContributionSetting => format!(
                                        "{} {}{}",
                                        format_number(*event.amount, locale),
                                        event
                                            .currency
                                            .expect("contribution events have a currency"),
                                        text.per_month
                                    ),
                                    AppliedAssetEventKind::ExpectedReturn => {
                                        format_percentage(*event.amount, locale)
                                    }
                                },
                            })
                            .collect(),
                    }
                })
                .collect(),
        })
        .collect()
}

fn format_duration(months: u32, locale: Locale) -> String {
    let years = months / 12;
    let remaining_months = months % 12;

    match locale {
        Locale::EnUs => match (years, remaining_months) {
            (0, 1) => "1 month".to_owned(),
            (0, _) => format!("{remaining_months} months"),
            (1, 0) => "1 year".to_owned(),
            (1, 1) => "1 year, 1 month".to_owned(),
            (1, _) => format!("1 year, {remaining_months} months"),
            (_, 0) => format!("{years} years"),
            (_, 1) => format!("{years} years, 1 month"),
            _ => format!("{years} years, {remaining_months} months"),
        },
        Locale::PtBr => match (years, remaining_months) {
            (0, 1) => "1 mês".to_owned(),
            (0, _) => format!("{remaining_months} meses"),
            (1, 0) => "1 ano".to_owned(),
            (1, 1) => "1 ano e 1 mês".to_owned(),
            (1, _) => format!("1 ano e {remaining_months} meses"),
            (_, 0) => format!("{years} anos"),
            (_, 1) => format!("{years} anos e 1 mês"),
            _ => format!("{years} anos e {remaining_months} meses"),
        },
    }
}

fn chart_x(index: usize, last_index: usize) -> String {
    let width = Decimal::from(CHART_RIGHT - CHART_LEFT);
    let position = if last_index == 0 {
        Decimal::from(CHART_LEFT)
    } else {
        Decimal::from(CHART_LEFT)
            + width * Decimal::from(index as u32) / Decimal::from(last_index as u32)
    };
    coordinate(position)
}

fn chart_y(value: Decimal, maximum: Decimal) -> String {
    let height = Decimal::from(CHART_BOTTOM - CHART_TOP);
    coordinate(Decimal::from(CHART_BOTTOM) - value / maximum * height)
}

fn chart_milestone_label_y(value: Decimal, maximum: Decimal) -> String {
    let line_y =
        Decimal::from(CHART_BOTTOM) - value / maximum * Decimal::from(CHART_BOTTOM - CHART_TOP);
    let offset = Decimal::from(12);
    let label_y = if line_y - offset <= Decimal::from(CHART_TOP) {
        line_y + offset + Decimal::from(6)
    } else {
        line_y - offset
    };
    coordinate(label_y)
}

fn coordinate(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

fn format_chart_axis_label(value: Decimal, locale: Locale) -> String {
    if value >= Decimal::from(1_000_000) {
        let suffix = match locale {
            Locale::EnUs => "M",
            Locale::PtBr => " mi",
        };
        format!(
            "{}{}",
            trim_trailing_decimal_zeros(
                format_number(value / Decimal::from(1_000_000), locale),
                locale
            ),
            suffix
        )
    } else if value >= Decimal::from(1_000) {
        let suffix = match locale {
            Locale::EnUs => "k",
            Locale::PtBr => " mil",
        };
        format!(
            "{}{}",
            trim_trailing_decimal_zeros(
                format_number(value / Decimal::from(1_000), locale),
                locale
            ),
            suffix
        )
    } else {
        trim_trailing_decimal_zeros(format_number(value, locale), locale)
    }
}

fn trim_trailing_decimal_zeros(mut value: String, locale: Locale) -> String {
    let decimal_separator = match locale {
        Locale::EnUs => '.',
        Locale::PtBr => ',',
    };
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with(decimal_separator) {
        value.pop();
    }
    value
}

fn format_plan_money(money: &PlanMoney<'_>, locale: Locale) -> String {
    format_number(money.plan_amount(), locale)
}

fn format_signed_number(amount: Decimal, locale: Locale) -> String {
    let sign = if amount >= Decimal::ZERO { "+" } else { "" };
    format!("{sign}{}", format_number(amount, locale))
}

fn format_percentage(rate: Decimal, locale: Locale) -> String {
    format!("{}%", format_decimal(rate * Decimal::from(100), 2, locale))
}

fn format_rounded_percentage(rate: Decimal, locale: Locale) -> String {
    format!("{}%", format_decimal(rate * Decimal::from(100), 0, locale))
}

fn format_number(amount: Decimal, locale: Locale) -> String {
    format_decimal(amount, 2, locale)
}

#[cfg(test)]
mod tests;

fn format_decimal(amount: Decimal, decimal_places: usize, locale: Locale) -> String {
    let formatted = format!(
        "{amount:.decimal_places$}",
        amount = amount.round_dp(decimal_places as u32)
    );
    let (sign, absolute) = formatted
        .strip_prefix('-')
        .map_or(("", formatted.as_str()), |absolute| ("-", absolute));
    let (whole, fraction) = absolute.split_once('.').unwrap_or((absolute, ""));
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

    if decimal_places == 0 {
        format!("{sign}{grouped_whole}")
    } else {
        format!("{sign}{grouped_whole}{decimal_separator}{fraction}")
    }
}
