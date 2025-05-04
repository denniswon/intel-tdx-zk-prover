use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Request not found")]
    NotFound,
    #[error("Request invalid")]
    Invalid,
    #[error("Request unauthorized")]
    Unauthorized,
}
