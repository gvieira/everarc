use super::*;
use crate::data_export::ProjectionExport;

#[test]
fn renders_inherited_event_rows_without_plan_currency_codes() {
    let mut config: Config =
        toml::from_str(include_str!("../../../everarc.toml")).expect("sample configuration parses");
    for scenario in &mut config.scenarios {
        scenario.selected = false;
    }
    config.scenarios[0].selected = true;
    config.validate().expect("sample configuration validates");
    let projection = PlanProjection::from(&config);
    let dashboard =
        DashboardPresentation::with_actual_balances(&config, &projection, config.display.locale);
    let html = render_dashboard(&dashboard).expect("dashboard renders");

    assert!(!html.contains("Car purchase -25.000,00"));
    assert!(!html.contains("<title>Everarc ·"));
    assert!(html.contains("<title>Everarc</title>"));
    assert!(html.contains("↗7,00%"));
    assert!(html.contains("+3.000,00 USD</span>"));
    assert!(!html.contains("−0,00 USD"));
    assert!(html.contains("class=\"chart-event-marker asset-line-0\""));
    assert!(html.contains("<span>Contribuição</span><strong>"));
    assert!(html.contains("<span>Retirada</span><strong>"));
    assert!(html.contains("<span>Renda passiva</span><strong>"));
    assert!(html.contains("aria-label=\"Rendimento anualizado da renda passiva\""));
    assert!(html.contains("Inflação anual: 3,00%"));
    assert!(html.contains("Ajustado pela inflação"));
    assert!(html.contains("type=\"checkbox\" checked data-living-cost-inflation-toggle"));
    assert!(html.contains(
        "class=\"cost-description-trigger\" tabindex=\"0\">Moradia<span class=\"cost-description-tooltip\" role=\"tooltip\">Aluguel, condomínio e manutenção.\nInflação anual: 6,00%</span>"
    ));
    assert!(html.contains(
        "data-living-cost-view=\"adjusted\">56%</span><span data-living-cost-view=\"today-money\" hidden>42%"
    ));
    assert!(html.contains(
        "data-living-cost-view=\"adjusted\">8.017,84</span><span data-living-cost-view=\"today-money\" hidden>2.500,00"
    ));
    assert!(html.contains(
        "data-living-cost-view=\"adjusted\">14.339,23</span><span data-living-cost-view=\"today-money\" hidden>6.000,00"
    ));
    assert!(html.contains("class=\"chart-asset-milestone asset-line-0\""));
    assert!(html.contains("data-chart-lifecycle-marker"));
    assert!(html.contains("chart-asset-lifecycle-start"));
    assert!(html.contains("chart-asset-lifecycle-end"));
    assert!(html.contains("data-chart-total-balance="));
    assert!(html.contains("data-chart-asset-balance="));
    assert!(html.contains("data-chart-asset-line=\"0\""));
    assert!(html.contains("data-chart-asset-index=\"0\""));
    assert!(html.contains("data-chart-asset-id=\"brokerage\" data-chart-asset-visual"));
    assert!(html.contains("data-chart-asset-visibility data-chart-asset-id=\"brokerage\""));
    assert!(html.contains("data-chart-assets-show-all"));
    assert!(html.contains("data-chart-assets-hide-all"));
    assert!(html.contains("class=\"chart-asset-lifecycle-change\">Iniciado"));
    assert!(html.contains("class=\"chart-asset-lifecycle-change\">Encerrado"));
    assert!(html.contains("data-chart-month-inspector aria-live=\"polite\""));
    assert!(html.contains("data-chart-month-summary"));
    assert!(html.contains("data-chart-month-details"));
    assert!(html.contains("data-chart-notable-month"));
    assert!(html.contains("data-chart-notable-month-navigation"));
    assert!(html.contains(".chart-card.is-month-pinned .chart-notable-month-navigation"));
    assert!(html.contains("data-chart-previous-notable-month"));
    assert!(html.contains("data-chart-next-notable-month"));
    assert!(html.contains("data-contribution-chart"));
    assert!(html.contains("data-contribution-bar"));
    assert!(html.contains("data-withdrawal-bar"));
    assert!(html.contains("data-net-flow-label="));
    assert!(html.contains("data-contribution-amount="));
    assert!(html.contains("data-contribution-hit-area"));
    assert!(html.contains("Fluxo mensal"));
    assert!(!html.contains("Contribuições mensais brutas"));
    assert!(html.contains("contribution-fill-0"));
    assert!(html.contains("data-contribution-hover-card"));
    assert!(html.contains("showContributionHover(chart, index)"));
    assert!(html.contains("percentageFormatter.format"));
    assert!(html.contains("const roundedFlowMaximum"));
    assert!(html.contains("chart.dataset.flowBaseline = flowBaseline"));
    assert!(html.contains(
        "hoverCard.replaceChildren(selected.querySelector(\"[data-chart-month-summary]\").cloneNode(true))"
    ));
    assert!(html.contains(
        "inspector.replaceChildren(selected.querySelector(\"[data-chart-month-details]\").cloneNode(true))"
    ));
    assert!(html.contains("pinnedMonthIndex: null"));
    assert!(html.contains("togglePinnedChartMonth(chart, pointerMonthIndex(event, area))"));
    assert!(html.contains("pinChartMonth(chart, nextIndex)"));
    assert!(html.contains("month.hasAttribute(\"data-chart-notable-month\")"));
    assert!(html.contains(
        "pinChartMonth(chart, Number(notableMonthButton.dataset.chartNotableMonthIndex))"
    ));
    assert!(html.contains("if (dragged)"));
    assert!(html.contains("pointerIsNearPinnedMonth(event, area)"));
    assert!(html.contains("Math.abs(event.clientX - pinnedX) <= 8"));
    assert!(html.contains("[data-chart-milestone-marker], [data-chart-lifecycle-marker]"));
    assert!(html.contains("const hiddenChartAssetIds = new Set()"));
    assert!(html.contains("applyAllChartAssetVisibility()"));
    assert!(html.contains("visual.classList.toggle(\"is-chart-asset-hidden\""));
    assert!(html.contains("data-chart-asset-active=\"True\""));
    assert!(html.contains("chartAssetActive.toLowerCase() === \"true\""));
    assert!(html.contains("Atingida em 04&#x2f;2043"));
    assert!(html.contains("data-scenario-plan-summary data-scenario-id=\"baseline\""));
    assert!(html.contains("data-safe-withdrawal-rate-toggle"));
    assert!(html.contains("Taxa de retirada segura"));
    assert!(html.contains("data-monthly-withdrawal="));
    assert!(html.contains("setPlanSummaryWithdrawalRate(card, toggle.checked)"));
    assert!(html.contains("data-scenario-default=\"true\""));
    assert!(html.contains(
        "Base <span class=\"scenario-selection-mark\" aria-label=\"Selecionado\">★</span><span class=\"scenario-description-tooltip\" role=\"tooltip\">Projeção com retornos, inflação e aportes esperados.</span>"
    ));
    assert!(html.contains(
        "<p class=\"scenario-description\">Projeção com retornos, inflação e aportes esperados.</p>"
    ));
    assert!(html.contains("scenario-selection-mark\" aria-label=\"Selecionado\">★</span>"));
    assert!(html.contains("data-passive-income=\"12.456,32\""));
    assert!(html.contains("data-passive-income=\"14.293,62\""));
    assert!(html.contains("<div class=\"conversion-rates\">"));
    assert!(html.contains("1 BTC = 85.000,00 USD"));
    assert!(html.contains("<span>Car purchase</span><strong>-25.000,00</strong>"));
    assert!(
        html.contains("<span>Increase contribution</span><strong>3.000,00 USD&#x2f;mês</strong>")
    );

    for value in html
        .split("<span class=\"chart-inspector-native\">")
        .skip(1)
    {
        let value = value
            .split("</span>")
            .next()
            .expect("native inspector span closes");
        assert!(!value.contains("USD"));
    }
    for value in html
        .split("<span class=\"chart-inspector-comparable\">")
        .skip(1)
    {
        let value = value
            .split("</span>")
            .next()
            .expect("comparable inspector span closes");
        assert!(!value.contains("USD"));
    }
    assert!(html.contains("BTC</span>"));
}

#[test]
fn writes_dashboard_data_as_pretty_json() {
    let output_directory = env::temp_dir().join(format!(
        "everarc-build-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir(&output_directory).expect("test output directory creates");

    let config_path = output_directory.join("everarc.toml");
    let html_path = output_directory.join("everarc.html");
    let data_path = output_directory.join("dashboard-data.json");
    fs::write(&config_path, include_str!("../../../everarc.toml"))
        .expect("sample configuration writes");

    build_once(&config_path, &html_path, Some(&data_path)).expect("dashboard builds");

    let config = Config::load(&config_path).expect("sample configuration loads");
    let projection = PlanProjection::from(&config);
    let data = fs::read_to_string(&data_path).expect("dashboard data reads");
    let actual: serde_json::Value = serde_json::from_str(&data).expect("dashboard data is JSON");
    let expected = ProjectionExport::with_actual_balances(&config, &projection);

    assert!(html_path.is_file());
    assert!(data.starts_with("{\n"));
    assert_eq!(actual["plan"]["currency"], "USD");
    assert_eq!(actual["plan"]["start"], "2026-01");
    assert_eq!(actual["scenarios"][0]["id"], "baseline");
    assert!(actual["scenarios"][0]["months"].is_array());
    assert!(actual["scenarios"][0]["months"][0]["assets"].is_array());
    assert!(actual["scenarios"][0]["outcome"]["end_balance"].is_string());
    assert!(actual["scenarios"][0]["events"].is_array());
    assert!(actual["scenarios"][0]["future_living_costs"]["costs"].is_array());
    assert!(actual.get("locale").is_none());
    assert!(actual.get("text").is_none());
    assert_eq!(
        actual,
        serde_json::to_value(expected).expect("projection export serializes")
    );

    fs::remove_dir_all(output_directory).expect("test output directory removes");
}
