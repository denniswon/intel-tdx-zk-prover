use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteError {
    #[error("Quote not found")]
    NotFound,
    #[error("Quote invalid")]
    Invalid,
    #[error("Quote unauthorized")]
    Unauthorized,
    #[error("Failed to update quote status on success")]
    UpdateStatusOnSuccess,
    #[error("Failed to update quote status on failure")]
    UpdateStatusOnFailure,
    #[error("Failed to submit proof")]
    SubmitProof,
    #[error("Failed to prove")]
    Prove,
    #[error("Failed to verify proof")]
    VerifyProof,
}
