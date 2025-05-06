use dotenvy;

pub fn init() {
    if std::env::var("ENV").unwrap_or("dev".to_string()) != "prod" {
        dotenvy::dotenv().unwrap();
    }
}

pub fn get(parameter: &str, default_value: Option<&str>) -> String {
    std::env::var(parameter)
        .unwrap_or_else(|_| default_value
            .unwrap_or_else(|| panic!("{} is not defined in the environment.", parameter))
            .to_string())
}
