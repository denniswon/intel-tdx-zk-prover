use dotenvy;

pub fn init() {
    unsafe {
        dotenvy::dotenv().unwrap_unchecked();
    }
}

pub fn get(parameter: &str) -> String {
    std::env::var(parameter)
        .unwrap_or_else(|_| panic!("{} is not defined in the environment.", parameter))
}
