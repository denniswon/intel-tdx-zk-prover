use dotenvy;

pub fn init() {
    dotenvy::dotenv().expect("Failed to load .env file");
}

pub fn get(parameter: &str) -> String {
    std::env::var(parameter)
        .unwrap_or_else(|_| panic!("{} is not defined in the environment.", parameter))
}
