use serde::Deserialize;

use crate::{amount::Amount, error::TransactionError};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionVariant {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// Unfortunately the csv crate does not support deserializing to more complex
// enum variants and we have to use a struct with a slightly more awkward type
// definition.
// This means that it is actually possible to read in a [`Transaction`]
// that is not actaully valid, for exmaple with variant = `TransactionVariant::Dispute` and
// amount = `Some(5.0)` which is illegal.
//
// It would be better if we could deserialize to something like:
// enum RowInput {
//     Transaction(Transaction),
//     DisputeOperation(DisputeOp),
// }
//
// Related issue: https://github.com/BurntSushi/rust-csv/issues/211
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub variant: TransactionVariant,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Amount>,
    #[serde(skip_deserializing)]
    pub disputed: bool,
    #[serde(skip_deserializing)]
    pub chargeback: bool,
}

impl Transaction {
    pub fn is_valid(&self) -> bool {
        match self.variant {
            TransactionVariant::Deposit | TransactionVariant::Withdrawal => self.amount.is_some(),
            _ => self.amount.is_none(),
        }
    }

    /// Check wether it is possible to dispute this transaction.
    ///
    /// It is only possible if it has not already been disputed and a chargeback
    /// has not happened.
    pub fn can_dispute(&self) -> Result<(), TransactionError> {
        if self.disputed {
            return Err(TransactionError::AlreadyDisputed);
        }
        if self.chargeback {
            return Err(TransactionError::TransactionChargedback);
        }
        Ok(())
    }

    /// Check wether it is possible to resolve or chargeback this transaction.
    ///
    /// It is only possible to resolve or chargeback a transaction if it has been
    /// disputed and a chargeback has not happened.
    pub fn can_resolve_or_chargeback(&self) -> Result<(), TransactionError> {
        if !self.disputed {
            return Err(TransactionError::NotDisputed);
        }
        if self.chargeback {
            return Err(TransactionError::TransactionChargedback);
        }
        Ok(())
    }
}
