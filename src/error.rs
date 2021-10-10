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
    #[error("An amount used in a transaction cannot be negative")]
    NegativeAmount,
    #[error("The transaction was not found")]
    TransactionNotFound,
    #[error("The transaction has been chargedback and not be updated")]
    TransactionChargedback,
    #[error("Cannot resolve a transaction that is not yet disputed")]
    NotDisputed,
    #[error("The transaction is already disputed")]
    AlreadyDisputed,
}
