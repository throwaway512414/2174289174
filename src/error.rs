use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum TransactionError {
    #[error("Account is locked")]
    LockedAccount,
    #[error("Cannot overwrite an existing transaction")]
    TransactionAlreadyExist,
    #[error("Cannot withdraw more money from an account than it has available")]
    InsufficientFunds,
    #[error("The transaction was not found")]
    TransactionNotFound,
    #[error("Cannot resolve a transaction that is not yet disputed")]
    NotDisputed,
    #[error("The transaction is already disputed")]
    AlreadyDisputed,
}
