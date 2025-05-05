use std::collections::HashMap;
use tarantool::log::TarantoolLogger;

pub fn setup_logger() {
    tarolog::set_default_logger_format(tarolog::Format::JsonTarantool(None));
    static LOGGER: TarantoolLogger = TarantoolLogger::new();

    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestRequest {
    pub some_string: String,
    pub some_int: u64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestResponse {
    pub request: TestRequest,
    pub path: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "OK".to_string(),
        }
    }
}
