use thiserror::Error;

use crate::Amount;

#[derive(Debug, PartialEq, Error)]
pub enum TransactionError {
    #[error("Account is locked")]
    LockedAccount,
    #[error("Cannot overwrite an existing transaction")]
    TransactionAlreadyExist,
    #[error("Insufficient funds for client `{client}` with available amount `{available}`. Attempt to withdraw `{amount_attempted}` failed.")]
    InsufficientFunds {
        client: u16,
        available: Amount,
        amount_attempted: Amount,
    },
    #[error("The transaction was not found")]
    TransactionNotFound,
    #[error("Cannot resolve a transaction that is not yet disputed")]
    NotDisputed,
    #[error("The transaction is already disputed")]
    AlreadyDisputed,
}
