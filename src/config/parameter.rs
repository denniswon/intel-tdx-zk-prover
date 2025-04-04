use dotenvy;

pub fn init() {
    dotenvy::dotenv().ok().expect("Failed to load .env file");
}

pub fn get(parameter: &str) -> String {
    let env_parameter = std::env::var(parameter)
        .expect(&format!("{} is not defined in the environment.", parameter));
    return env_parameter;
}
