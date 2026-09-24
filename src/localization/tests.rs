use rust_decimal::Decimal;

use super::*;
use crate::{config::Config, projection::PlanProjection};

fn config(source: &str) -> Config {
    let mut config: Config = toml::from_str(source).expect("test configuration parses");
    config.validate().expect("test configuration validates");
    config
}

#[test]
fn shared_chart_scale_includes_root_total_balance_milestones() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-02"

[[milestones]]
id = "goal"
name = "Goal"
target = "1000"

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
id = "initial-balance"
name = "Initial balance"
currency = "USD"
value = "100"

[[scenarios.milestones]]
id = "cash-goal"
name = "Cash goal"
asset_id = "cash"
target = "1500"
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);

    assert_eq!(dashboard.chart.y_ticks.last().unwrap().label, "1.5k");
    assert_eq!(dashboard.chart.milestones[0].line_y, "195.00");
    assert!(
        dashboard.scenarios[0].chart_assets[0]
            .milestone_markers
            .is_empty()
    );
    assert_eq!(dashboard.chart.x_grid, vec!["80", "950"]);
    assert_eq!(
        dashboard.scenarios[0].total_balance_milestones[0].timing,
        "Not reached by 02/2026"
    );
    assert_eq!(
        dashboard.scenarios[0].asset_milestones[0].timing,
        "Not reached by 02/2026"
    );
    assert_eq!(dashboard.scenarios[0].chart_assets.len(), 1);
    assert!(
        dashboard.scenarios[0].chart_assets[0]
            .path
            .starts_with("M 80 ")
    );
    assert_eq!(dashboard.scenarios[0].chart_months.len(), 2);
    assert_eq!(
        dashboard.scenarios[0].chart_months[0].assets[0].comparable_plan_balance,
        None
    );
}

#[test]
fn chart_month_inspector_includes_converted_asset_values() {
    let config = config(
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
monthly_contribution = { amount = "0", currency = "USD" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "BTC"
value = "0.25"

[[scenarios.events]]
id = "start-contribution"
name = "Start contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "bitcoin"
monthly_contribution = { amount = "0.2", currency = "BTC" }

[[scenarios.events]]
id = "ignored-return"
name = "Ignored return"
type = "set_annual_expected_return"
date = "2026-01"
asset_id = "bitcoin"
rate = "0.5"

[[scenarios.events]]
id = "rebalance"
name = "Rebalance"
type = "asset_adjustment"
date = "2026-01"
asset_id = "bitcoin"
amount = "-0.1"

[[scenarios.events]]
id = "percentage-increase"
name = "Percentage increase"
type = "adjust_monthly_contribution"
date = "2026-01"
asset_id = "bitcoin"
monthly_contribution = { rate = "0" }
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    assert_eq!(dashboard.plan.duration, "1 month");
    assert_eq!(dashboard.plan.conversion_rates[0].rate, "10,000.00");
    assert_eq!(dashboard.plan.conversion_rates[0].from.to_string(), "BTC");
    assert_eq!(dashboard.plan.conversion_rates[0].to.to_string(), "USD");
    let asset = &dashboard.scenarios[0].chart_months[0].assets[0];

    assert_eq!(asset.id, "bitcoin");
    assert_eq!(asset.native_balance, "0.36");
    assert_eq!(asset.annual_expected_return, "50.00%");
    assert_eq!(asset.monthly_contribution, "0.20");
    assert!(asset.has_monthly_contribution);
    assert_eq!(asset.monthly_contribution_currency.to_string(), "BTC");
    assert!(!asset.has_monthly_withdrawal);
    assert_eq!(asset.passive_income, "85.92");
    assert_eq!(asset.y, "112.27");
    assert_eq!(asset.color_index, 0);
    assert_eq!(asset.comparable_plan_balance.as_deref(), Some("3,585.92"));
    assert_eq!(
        dashboard.scenarios[0].chart_months[0].passive_income,
        "85.92"
    );
    assert!(dashboard.scenarios[0].chart_months[0].has_monthly_contributions);
    assert_eq!(
        dashboard.scenarios[0].chart_months[0].monthly_contributions,
        "2,000.00"
    );
    assert_eq!(
        dashboard.scenarios[0].chart_months[0]
            .monthly_contributions_amount
            .parse::<Decimal>()
            .unwrap(),
        Decimal::new(2000, 0)
    );
    assert!(!dashboard.scenarios[0].chart_months[0].has_monthly_withdrawals);
    assert_eq!(asset.events.len(), 4);
    assert_eq!(asset.events[0].name, "Start contribution");
    assert_eq!(asset.events[0].value, "0.20 BTC/month");
    assert_eq!(asset.events[1].name, "Ignored return");
    assert_eq!(asset.events[1].value, "50.00%");
    assert_eq!(asset.events[2].name, "Rebalance");
    assert_eq!(asset.events[2].value, "-0.10");
    assert_eq!(asset.events[3].name, "Percentage increase");
    assert_eq!(asset.events[3].value, "+0.00%");
    assert_eq!(dashboard.scenarios[0].chart_assets.len(), 1);
    assert_eq!(dashboard.scenarios[0].chart_assets[0].id, "bitcoin");
    assert_eq!(
        dashboard.scenarios[0].chart_assets[0].event_markers.len(),
        1
    );
    assert!(
        dashboard.scenarios[0].chart_assets[0]
            .path
            .starts_with("M 80 ")
    );
}

#[test]
fn monthly_contribution_totals_follow_flows_conversion_and_lifecycles() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

[[conversion_rates]]
from = "BTC"
to = "USD"
rate = "10000"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = { amount = "100", currency = "USD" }
monthly_withdrawal = { amount = "50", currency = "USD" }

[[scenarios.assets.holdings]]
id = "cash-opening"
name = "Cash opening"
currency = "USD"
value = "0"

[[scenarios.assets]]
id = "bitcoin"
name = "Bitcoin"
currency = "BTC"
annual_expected_return = "0"
monthly_contribution = { amount = "0.01", currency = "BTC" }
starts = "2026-02"
ends = "2026-02"

[[scenarios.assets.holdings]]
id = "bitcoin-opening"
name = "Bitcoin opening"
currency = "BTC"
value = "0"

[[scenarios.events]]
id = "set-cash-contribution"
name = "Set cash contribution"
type = "set_monthly_contribution"
date = "2026-02"
asset_id = "cash"
monthly_contribution = { amount = "200", currency = "USD" }

[[scenarios.events]]
id = "increase-bitcoin-contribution"
name = "Increase bitcoin contribution"
type = "adjust_monthly_contribution"
date = "2026-02"
asset_id = "bitcoin"
monthly_contribution = { amount = "0.01", currency = "BTC" }
recurrence = { every = 1, unit = "months" }

[[scenarios.events]]
id = "increase-cash-contribution"
name = "Increase cash contribution"
type = "adjust_monthly_contribution"
date = "2026-03"
asset_id = "cash"
monthly_contribution = { rate = "0.5" }

[[scenarios.events]]
id = "cash-purchase"
name = "Cash purchase"
type = "asset_adjustment"
date = "2026-02"
asset_id = "cash"
amount = "-1000"
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    let months = &dashboard.scenarios[0].chart_months;

    assert_eq!(months[0].monthly_contributions, "100.00");
    assert_eq!(months[1].monthly_contributions, "400.00");
    assert_eq!(months[1].assets[0].plan_monthly_contribution, "200.00");
    assert_eq!(months[1].assets[0].plan_monthly_contribution_amount, "200");
    assert_eq!(months[1].assets[0].plan_monthly_withdrawal, "50.00");
    assert_eq!(months[1].assets[0].plan_monthly_withdrawal_amount, "50");
    assert_eq!(
        months[1].assets[1]
            .plan_monthly_contribution_amount
            .parse::<Decimal>()
            .unwrap(),
        Decimal::new(200, 0)
    );
    assert_eq!(months[2].monthly_contributions, "300.00");
}

#[test]
fn presents_every_recurring_asset_adjustment_occurrence() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-03"

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
id = "opening"
name = "Opening"
currency = "USD"
value = "0"

[[scenarios.events]]
id = "monthly-purchase"
name = "Monthly purchase"
type = "asset_adjustment"
date = "2026-01"
asset_id = "cash"
amount = "0"
recurrence = { every = 1, unit = "months" }
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    let scenario = &dashboard.scenarios[0];

    assert_eq!(scenario.chart_assets[0].event_markers.len(), 3);
    assert!(
        scenario
            .chart_months
            .iter()
            .all(|month| month.assets[0].events.len() == 1)
    );
}

#[test]
fn localizes_month_displays() {
    let month: Month = "2043-05".parse().unwrap();

    assert_eq!(format_month(month, Locale::EnUs), "05/2043");
    assert_eq!(format_month(month, Locale::PtBr), "05/2043");
}

#[test]
fn adds_vertical_chart_grid_lines_at_five_year_calendar_marks() {
    let config = config(include_str!("../../everarc.toml"));
    let projection = PlanProjection::from(&config);
    let months = &projection.scenarios[0].total_net_worth;

    let grid = chart_x_grid(months);
    assert_eq!(grid.len(), 6);
    assert!(grid.contains(&chart_x(48, months.len() - 1)));
    assert!(grid.contains(&chart_x(228, months.len() - 1)));
}

#[test]
fn formats_compact_localized_chart_axis_labels() {
    assert_eq!(
        format_chart_axis_label(Decimal::new(2_500_000, 0), Locale::PtBr),
        "2,5 mi"
    );
    assert_eq!(
        format_chart_axis_label(Decimal::new(500_000, 0), Locale::EnUs),
        "500k"
    );
}

#[test]
fn rounds_chart_scales_to_nice_intervals() {
    assert_eq!(
        nice_chart_scale(Decimal::new(207_185_851, 2)),
        ChartScale {
            maximum: Decimal::new(2_500_000, 0),
            step: Decimal::new(500_000, 0),
        }
    );
}

#[test]
fn presents_living_cost_descriptions_and_shares() {
    let config = config(
        r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-12"

[[future_living_costs]]
id = "housing"
name = "Housing"
description = "Rent and maintenance."
annual_inflation = "0.1"
monthly_cost = "2500"

[[future_living_costs]]
id = "food"
name = "Food"
monthly_cost = "500"

[[scenarios]]
id = "base"
name = "Base"
description = "Long-term plan."
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
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    assert_eq!(dashboard.scenarios[0].description, Some("Long-term plan."));
    let costs = &dashboard.scenarios[0].future_living_costs.costs;

    assert_eq!(
        costs[0].tooltip.as_deref(),
        Some("Rent and maintenance.\nAnnual inflation: 10.00%")
    );
    assert_eq!(costs[0].nominal_monthly_cost, "2,750.00");
    assert_eq!(costs[0].nominal_share, "85%");
    assert_eq!(costs[0].today_money_share, "83%");
    assert_eq!(costs[1].tooltip.as_deref(), Some("Annual inflation: 0.00%"));
    assert_eq!(costs[1].nominal_share, "15%");
    assert_eq!(costs[1].today_money_share, "17%");
}

#[test]
fn localizes_plan_duration() {
    let config = config(
        r#"
[display]
locale = "pt-BR"

[plan]
currency = "BRL"
start = "2026-01"
end = "2026-01"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "BRL"
annual_expected_return = "0"
monthly_contribution = { amount = "0", currency = "BRL" }

[[scenarios.assets.holdings]]
id = "initial-balance"
name = "Initial balance"
currency = "BRL"
value = "0"
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);

    assert_eq!(dashboard.plan.duration, "1 mês");
    assert_eq!(dashboard.scenarios[0].end_of_plan_passive_income, "0,00");
    assert_eq!(dashboard.text.annual_return, "Retorno anual");
    assert_eq!(dashboard.text.monthly_contribution, "Contribuição mensal");
    assert_eq!(dashboard.text.per_month, "/mês");
    assert_eq!(dashboard.text.pinned, "Fixado");
    assert_eq!(dashboard.text.show_asset, "Mostrar");
    assert_eq!(dashboard.text.hide_asset, "Ocultar");
    assert_eq!(dashboard.text.show_all_assets, "Mostrar todos");
    assert_eq!(dashboard.text.hide_all_assets, "Ocultar todos");
    assert_eq!(
        dashboard.text.pin_instruction,
        "Clique em um mês para fixar/desafixar"
    );
}

#[test]
fn cycles_chart_colors_after_the_full_asset_palette() {
    let mut source = String::from(
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
"#,
    );
    for index in 0..9 {
        source.push_str(&format!(
            r#"
[[scenarios.assets]]
id = "asset-{index}"
name = "Asset {index}"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = {{ amount = "0", currency = "USD" }}

[[scenarios.assets.holdings]]
id = "holding-{index}"
name = "Holding {index}"
currency = "USD"
value = "{index}"
"#
        ));
    }

    let config = config(&source);
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    let colors = dashboard.scenarios[0]
        .chart_assets
        .iter()
        .map(|asset| asset.color_index)
        .collect::<Vec<_>>();

    assert_eq!(colors, [0, 1, 2, 3, 4, 5, 6, 7, 0]);
    assert_eq!(
        dashboard.scenarios[0].chart_months[0].assets[8].color_index,
        0
    );
}
