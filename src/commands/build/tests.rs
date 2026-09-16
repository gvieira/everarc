use super::*;

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
    let dashboard = DashboardPresentation::new(&projection, config.display.locale);
    let html = render_dashboard(&dashboard).expect("dashboard renders");

    assert!(!html.contains("Car purchase -25.000,00"));
    assert!(!html.contains("<title>Everarc ·"));
    assert!(html.contains("<title>Everarc</title>"));
    assert!(html.contains("↗ 7,00%"));
    assert!(html.contains("+ 3.000,00&#x2f;mês"));
    assert!(html.contains("class=\"chart-event-marker asset-line-0\""));
    assert!(html.contains("<span>Renda passiva</span><strong>"));
    assert!(html.contains("Inflação anual: 3,00%"));
    assert!(html.contains("Inflação de 3,00%"));
    assert!(html.contains("type=\"checkbox\" checked data-living-cost-inflation-toggle"));
    assert!(html.contains(
        "data-living-cost-value=\"adjusted\">4.515,28</span><span data-living-cost-value=\"today-money\" hidden>2.500,00"
    ));
    assert!(html.contains(
        "data-living-cost-value=\"adjusted\">10.836,67</span><span data-living-cost-value=\"today-money\" hidden>6.000,00"
    ));
    assert!(html.contains("class=\"asset-target-line asset-line-0\""));
    assert!(html.contains("data-scenario-plan-summary data-scenario-id=\"baseline\""));
    assert!(html.contains("data-scenario-default=\"true\""));
    assert!(html.contains("scenario-selection-mark\" aria-label=\"Selecionado\">★</span>"));
    assert!(html.contains("<p class=\"passive-income\">12.429,01</p>"));
    assert!(html.contains("<p class=\"passive-income\">14.578,81</p>"));
    assert!(html.contains("<div class=\"conversion-rates\">"));
    assert!(html.contains("1 BTC = 85.000,00 USD"));
    assert!(html.contains("<span>Car purchase</span><strong>-25.000,00</strong>"));
    assert!(html.contains("<span>Increase contribution</span><strong>3.000,00&#x2f;mês</strong>"));

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
