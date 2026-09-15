use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    config::{Currency, Locale, Month},
    projection::{
        AppliedAssetEventKind, AssetMilestoneProjection, PlanMoney, PlanProjection,
        TotalBalanceMilestoneProjection,
    },
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
    pub total_balance_milestones: Vec<DashboardMilestonePresentation<'projection>>,
    pub chart: DashboardChartPresentation<'projection>,
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
    pub total_net_worth: &'static str,
    pub change_since_first_projected_month: &'static str,
    pub projected_change: &'static str,
    pub change_from_zero_unavailable: &'static str,
    pub projection: &'static str,
    pub month: &'static str,
    pub total: &'static str,
    pub assets: &'static str,
    pub monthly_return: &'static str,
    pub monthly_contribution: &'static str,
    pub per_month: &'static str,
    pub total_net_worth_legend: &'static str,
    pub goals_legend: &'static str,
}

#[derive(Serialize)]
pub struct DashboardPlanPresentation<'projection> {
    pub currency: &'projection Currency,
    pub start: Month,
    pub end: Month,
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
    pub net_worth: DashboardNetWorthPresentation,
    pub chart_path: String,
    pub chart_assets: Vec<DashboardChartAssetLinePresentation<'projection>>,
    pub chart_asset_milestones: Vec<DashboardChartAssetMilestone<'projection>>,
    pub chart_months: Vec<DashboardChartMonthPresentation<'projection>>,
    pub future_living_costs: DashboardFutureLivingCostsPresentation<'projection>,
    pub asset_milestones: Vec<DashboardMilestonePresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartAssetLinePresentation<'projection> {
    pub name: &'projection str,
    pub path: String,
    pub color_index: usize,
    pub event_markers: Vec<DashboardChartEventMarker>,
}

#[derive(Serialize)]
pub struct DashboardChartEventMarker {
    pub x: String,
    pub y: String,
}

#[derive(Serialize)]
pub struct DashboardChartAssetMilestone<'projection> {
    pub name: &'projection str,
    pub line_y: String,
    pub label_y: String,
    pub color_index: usize,
}

#[derive(Serialize)]
pub struct DashboardChartMonthPresentation<'projection> {
    pub month: String,
    pub x: String,
    pub y: String,
    pub total: String,
    pub assets: Vec<DashboardChartAssetPresentation<'projection>>,
}

#[derive(Serialize)]
pub struct DashboardChartAssetPresentation<'projection> {
    pub name: &'projection str,
    pub native_balance: String,
    pub monthly_expected_return: String,
    pub monthly_contribution: String,
    pub is_plan_currency: bool,
    pub y: String,
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
pub struct DashboardNetWorthPresentation {
    pub end_total: String,
    pub absolute_change: String,
    pub percentage_change: Option<String>,
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
                .map(|scenario| chart_x_labels(&scenario.total_net_worth))
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
            chart,
            scenarios: projection
                .scenarios
                .iter()
                .map(|scenario| DashboardScenarioPresentation {
                    id: scenario.id(),
                    name: scenario.name(),
                    net_worth: DashboardNetWorthPresentation::from_scenario(scenario, locale),
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
                        })
                        .collect(),
                    chart_asset_milestones: scenario
                        .asset_milestones
                        .iter()
                        .filter_map(|milestone| {
                            scenario
                                .assets
                                .iter()
                                .position(|asset| asset.id() == milestone.asset_id())
                                .map(|color_index| DashboardChartAssetMilestone {
                                    name: milestone.name(),
                                    line_y: chart_y(
                                        milestone.target.plan_amount(),
                                        chart_scale.maximum,
                                    ),
                                    label_y: chart_milestone_label_y(
                                        milestone.target.plan_amount(),
                                        chart_scale.maximum,
                                    ),
                                    color_index,
                                })
                        })
                        .collect(),
                    chart_months: chart_months(
                        scenario,
                        chart_scale.maximum,
                        locale,
                        projection.plan.currency(),
                    ),
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

impl DashboardNetWorthPresentation {
    fn from_scenario(scenario: &crate::projection::ScenarioProjection<'_>, locale: Locale) -> Self {
        let first = scenario
            .total_net_worth
            .first()
            .map(|month| month.balance)
            .unwrap_or(Decimal::ZERO);
        let end = scenario
            .total_net_worth
            .last()
            .map(|month| month.balance)
            .unwrap_or(Decimal::ZERO);
        let change = end - first;

        Self {
            end_total: format_number(end, locale),
            absolute_change: format_signed_number(change, locale),
            percentage_change: (first != Decimal::ZERO)
                .then(|| format_signed_percent(change / first * Decimal::from(100), locale)),
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
                total_net_worth: "Total net worth",
                change_since_first_projected_month: "since first projected month",
                projected_change: "projected change",
                change_from_zero_unavailable: "Change unavailable from a zero first month",
                projection: "Projection",
                month: "Month",
                total: "Total",
                assets: "Assets",
                monthly_return: "Monthly return",
                monthly_contribution: "Monthly contribution",
                per_month: "/month",
                total_net_worth_legend: "Total net worth",
                goals_legend: "Goals",
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
                total_net_worth: "Patrimônio líquido total",
                change_since_first_projected_month: "desde o primeiro mês projetado",
                projected_change: "variação projetada",
                change_from_zero_unavailable: "Variação indisponível a partir de um primeiro mês zerado",
                projection: "Projeção",
                month: "Mês",
                total: "Total",
                assets: "Ativos",
                monthly_return: "Retorno mensal",
                monthly_contribution: "Contribuição mensal",
                per_month: "/mês",
                total_net_worth_legend: "Patrimônio líquido total",
                goals_legend: "Metas",
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
            label: month.month.to_string(),
        });
        labels
    })
}

fn chart_x_grid(months: &[crate::projection::TotalNetWorthMonthProjection]) -> Vec<String> {
    let last_index = months.len().saturating_sub(1);
    (0..months.len())
        .map(|index| chart_x(index, last_index))
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
) -> Vec<DashboardChartMonthPresentation<'projection>> {
    let last_index = scenario.total_net_worth.len().saturating_sub(1);
    scenario
        .total_net_worth
        .iter()
        .enumerate()
        .map(|(index, total)| DashboardChartMonthPresentation {
            month: total.month.to_string(),
            x: chart_x(index, last_index),
            y: chart_y(total.balance, maximum),
            total: format_number(total.balance, locale),
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
                        monthly_expected_return: format_percentage(
                            balance.monthly_expected_return,
                            locale,
                        ),
                        monthly_contribution: format_number(balance.monthly_contribution, locale),
                        is_plan_currency: asset.currency() == plan_currency,
                        y: chart_y(balance.plan_balance, maximum),
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
                                        "{}{}",
                                        format_number(*event.amount, locale),
                                        DashboardText::for_locale(locale).per_month
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

fn format_signed_percent(percent: Decimal, locale: Locale) -> String {
    let sign = if percent >= Decimal::ZERO { "+" } else { "" };
    format!("{sign}{}%", format_decimal(percent, 1, locale))
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
    let (whole, fraction) = absolute
        .split_once('.')
        .expect("a Decimal formatted with decimal places always has a decimal point");
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
