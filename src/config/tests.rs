use super::*;

#[test]
fn formats_and_chains_configuration_read_errors() {
    let error = ConfigError::Read {
        path: PathBuf::from("missing.toml"),
        source: io::Error::new(io::ErrorKind::NotFound, "not found"),
    };

    assert_eq!(
        error.to_string(),
        "failed to read configuration file `missing.toml`: not found"
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn rejects_conversion_rates_configured_in_both_directions() {
    let config: Config = toml::from_str(
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
rate = "100000"

[[conversion_rates]]
from = "USD"
to = "BTC"
rate = "0.00001"

[[scenarios]]
id = "base"
name = "Base"
annual_inflation = "0"
assets = []
"#,
    )
    .expect("configuration parses");

    assert!(matches!(
        config.validate(),
        Err(ConfigError::DuplicateConversionRate { .. })
    ));
}
