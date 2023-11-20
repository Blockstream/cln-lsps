use cln_plugin::options::{ConfigOption, Value};

pub fn lsps1_info_website() -> ConfigOption {
    ConfigOption::new(
        "lsps1.info.website",
        Value::OptString,
        "The website advertised in LSPS1"
    )
}

