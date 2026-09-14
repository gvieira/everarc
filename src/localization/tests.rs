use rust_decimal::Decimal;

use super::*;
use crate::{config::Config, projection::PlanProjection};

fn config(source: &str) -> Config {
    let config: Config = toml::from_str(source).expect("test configuration parses");
    config.validate().expect("test configuration validates");
    config
}

#[test]
fn shared_chart_scale_includes_root_total_balance_milestones() {
    let config = config(
        r#"
version = 1

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
initial_value = "100"
monthly_expected_return = "0"
monthly_contribution = "10"
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);

    assert_eq!(dashboard.chart.y_ticks.last().unwrap().label, "1k");
    assert_eq!(dashboard.chart.milestones[0].line_y, "75.0");
    assert_eq!(dashboard.chart.x_grid, vec!["80", "950"]);
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
version = 1

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
initial_value = "0.25"
monthly_expected_return = "0"
monthly_contribution = "0"

[[scenarios.events]]
id = "start-contribution"
name = "Start contribution"
type = "set_monthly_contribution"
date = "2026-01"
asset_id = "bitcoin"
amount = "0.2"

[[scenarios.events]]
id = "ignored-return"
name = "Ignored return"
type = "set_monthly_expected_return"
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
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    let asset = &dashboard.scenarios[0].chart_months[0].assets[0];

    assert_eq!(asset.native_balance, "0.48");
    assert_eq!(asset.monthly_expected_return, "50.00%");
    assert_eq!(asset.monthly_contribution, "0.20");
    assert_eq!(asset.y, "93.00");
    assert_eq!(asset.color_index, 0);
    assert_eq!(asset.comparable_plan_balance.as_deref(), Some("4,750.00"));
    assert_eq!(asset.events.len(), 3);
    assert_eq!(asset.events[0].name, "Start contribution");
    assert_eq!(asset.events[0].value, "0.20/month");
    assert_eq!(asset.events[1].name, "Ignored return");
    assert_eq!(asset.events[1].value, "50.00%");
    assert_eq!(asset.events[2].name, "Rebalance");
    assert_eq!(asset.events[2].value, "-0.10");
    assert_eq!(dashboard.scenarios[0].chart_assets.len(), 1);
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
        nice_chart_scale(Decimal::new(2_071_858_51, 2)),
        ChartScale {
            maximum: Decimal::new(2_500_000, 0),
            step: Decimal::new(500_000, 0),
        }
    );
}

#[test]
fn zero_first_month_has_no_percentage_change() {
    let config = config(
        r#"
version = 1

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
initial_value = "0"
monthly_expected_return = "0"
monthly_contribution = "0"
"#,
    );
    let projection = PlanProjection::from(&config);
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);

    assert_eq!(dashboard.scenarios[0].net_worth.end_total, "0,00");
    assert!(dashboard.scenarios[0].net_worth.percentage_change.is_none());
    assert_eq!(dashboard.text.monthly_return, "Retorno mensal");
    assert_eq!(dashboard.text.monthly_contribution, "Contribuição mensal");
    assert_eq!(dashboard.text.per_month, "/mês");
}
