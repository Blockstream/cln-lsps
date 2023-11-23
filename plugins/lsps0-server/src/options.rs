use cln_plugin::options::{ConfigOption, Value};

pub fn lsps1_info_website() -> ConfigOption {
    ConfigOption::new(
        "lsps1_info_website",
        Value::OptString,
        "The website advertised in LSPS1",
    )
}
