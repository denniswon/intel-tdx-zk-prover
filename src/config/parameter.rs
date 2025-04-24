use dotenvy;

pub fn init() {
    if std::env::var("ENV").unwrap_or("dev".to_string()) != "prod" {
        dotenvy::dotenv().unwrap();
    }
}

pub fn get(parameter: &str) -> String {
    std::env::var(parameter)
        .unwrap_or_else(|_| panic!("{} is not defined in the environment.", parameter))
}
