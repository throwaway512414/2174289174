use std::collections::HashMap;

use crate::{
    account::Account,
    error::TransactionError,
    transaction::{Transaction, TransactionVariant},
};

#[derive(Debug, Default)]
pub struct PaymentEngine {
    transactions: HashMap<u32, Transaction>,
    accounts: HashMap<u16, Account>,
}

impl PaymentEngine {
    /// Inserts a new [`Transaction`] to the [`PaymentEngine`].
    ///
    /// Returns a [`TransactionError`] if it could not be inserted.
    ///
    /// # Examples
    ///
    /// ```
    /// use randomlib::{Amount, PaymentEngine, Transaction, TransactionVariant};
    ///
    /// let mut engine = PaymentEngine::default();
    /// let tx = Transaction {
    ///    tx: 1,
    ///    amount: Some(Amount::new(104, 1).unwrap()),
    ///    client: 1,
    ///    disputed: false,
    ///    variant: TransactionVariant::Deposit,
    /// };
    /// assert!(engine.insert(tx).is_ok());
    /// ```
    // TODO: README
    // TODO: read through paper again
    pub fn insert(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        let account = self
            .accounts
            .entry(tx.client)
            // Or insert the Account if it does not exist already
            .or_insert_with(|| Account::new(tx.client));

        match tx.variant {
            TransactionVariant::Deposit | TransactionVariant::Withdrawal => {
                // Dont allow overwriting an existing transaction
                if self.transactions.get(&tx.tx).is_some() {
                    return Err(TransactionError::TransactionAlreadyExist);
                }

                // SAFETY: We knnow that when `variant` is `TransactionVariant::Deposit` or
                // `TransactionVariant::Withdrawal` that the amount is Some.
                let amount = tx.amount.unwrap();

                account.transaction(&tx.variant, amount)?;
                self.transactions.insert(tx.tx, tx);
            }
            TransactionVariant::Dispute => {
                let disputed_tx = self
                    .transactions
                    .get_mut(&tx.tx)
                    .ok_or(TransactionError::TransactionNotFound)?;

                if disputed_tx.client != tx.client {
                    return Err(TransactionError::TransactionNotFound);
                }

                if disputed_tx.disputed {
                    return Err(TransactionError::AlreadyDisputed);
                }

                // SAFETY: We knnow that `disputed_tx` has `variant` with value
                // `TransactionVariant::Deposit` or `TransactionVariant::Withdrawal`.
                // This means that `amount` is Some.
                let disputed_amount = disputed_tx.amount.unwrap();

                account.transaction(&tx.variant, disputed_amount)?;
                disputed_tx.disputed = true;
            }
            TransactionVariant::Resolve | TransactionVariant::Chargeback => {
                let disputed_tx = self
                    .transactions
                    .get_mut(&tx.tx)
                    .ok_or(TransactionError::TransactionNotFound)?;

                if disputed_tx.client != tx.client {
                    return Err(TransactionError::TransactionNotFound);
                }

                if !disputed_tx.disputed {
                    return Err(TransactionError::NotDisputed);
                }

                // SAFETY: We knnow that `disputed_tx` has `variant` with value
                // `TransactionVariant::Deposit` or `TransactionVariant::Withdrawal`.
                // This means that `amount` is Some.
                let disputed_amount = disputed_tx.amount.unwrap();

                account.transaction(&tx.variant, disputed_amount)?;
                disputed_tx.disputed = false;
            }
        }

        Ok(())
    }

    pub fn accounts(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount::Amount;

    #[test]
    fn simple_deposit() {
        let mut engine = PaymentEngine::default();

        let amount = Amount::new(22, 1).unwrap();
        let client = 1;
        let deposit = Transaction {
            tx: 1,
            amount: Some(amount),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());
        assert_eq!(engine.accounts.len(), 1);
        assert_eq!(engine.transactions.len(), 1);
        // Check account
        let account = engine.accounts.get(&client).unwrap();
        assert_eq!(account.available(), amount);
        assert_eq!(account.total(), amount);
        assert_eq!(account.held(), Amount::zero());
        // Check transaction
        let tx = engine.transactions.get(&1).unwrap();
        assert_eq!(tx.amount, Some(amount));
        assert_eq!(tx.client, client);
    }

    #[test]
    fn simple_withdrawal() {
        let mut engine = PaymentEngine::default();

        let amount = Amount::new(22, 1).unwrap();
        let client = 1;
        let deposit = Transaction {
            tx: 1,
            amount: Some(amount),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        let withdrawal = Transaction {
            tx: 2,
            amount: Some(amount),
            client,
            disputed: false,
            variant: TransactionVariant::Withdrawal,
        };
        assert!(engine.insert(withdrawal).is_ok());

        assert_eq!(engine.accounts.len(), 1);
        assert_eq!(engine.transactions.len(), 2);
        // Check account
        let account = engine.accounts.get(&client).unwrap();
        assert_eq!(account.available(), Amount::zero());
        assert_eq!(account.total(), Amount::zero());
        assert_eq!(account.held(), Amount::zero());
    }

    #[test]
    fn reject_too_large_withdrawal() {
        let mut engine = PaymentEngine::default();

        let mut amount = Amount::new(22, 1).unwrap();
        let client = 1;
        let deposit = Transaction {
            tx: 1,
            amount: Some(amount),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        amount += Amount::new(1, 1).unwrap();

        let withdrawal = Transaction {
            tx: 2,
            // Trying to withdraw an amount larger than the amount deposited
            amount: Some(amount),
            client,
            disputed: false,
            variant: TransactionVariant::Withdrawal,
        };
        assert_eq!(
            engine.insert(withdrawal).unwrap_err(),
            TransactionError::InsufficientFunds
        );
    }

    #[test]
    fn reject_transaction_overwrite() {
        let mut engine = PaymentEngine::default();

        let tx = 1;
        let client = 1;
        let deposit = Transaction {
            tx,
            amount: Some(Amount::zero()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());
        let deposit = Transaction {
            // Trying to use the same `tx` as in the previous transaction
            tx,
            amount: Some(Amount::zero()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert_eq!(
            engine.insert(deposit).unwrap_err(),
            TransactionError::TransactionAlreadyExist
        );
    }

    #[test]
    fn chargeback() {
        let mut engine = PaymentEngine::default();

        let client = 1;

        // Deposit
        let deposit = Transaction {
            tx: 1,
            amount: Some(Amount::new(10, 0).unwrap()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        let dispute = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Dispute,
        };
        assert!(engine.insert(dispute).is_ok());
        let chargeback = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Chargeback,
        };
        assert!(engine.insert(chargeback).is_ok());
        let account_after_chargeback = engine.accounts.get(&client).unwrap();

        // Check that deposit has been reversed and that everything is back to zero
        assert_eq!(account_after_chargeback.available(), Amount::zero());
        assert_eq!(account_after_chargeback.total(), Amount::zero());
        assert_eq!(account_after_chargeback.held(), Amount::zero());
        // Chargeback should lock account
        assert!(account_after_chargeback.locked());
    }

    #[test]
    fn resolved_dispute() {
        let mut engine = PaymentEngine::default();

        let client = 1;

        // Deposit
        let deposit = Transaction {
            tx: 1,
            amount: Some(Amount::new(10, 0).unwrap()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        // Backup state of account at this point to compare after dispute is resolved
        let accounts = engine.accounts.clone();
        let account_before_dispute = accounts.get(&client).unwrap();

        let dispute = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Dispute,
        };
        assert!(engine.insert(dispute).is_ok());
        let chargeback = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Resolve,
        };
        assert!(engine.insert(chargeback).is_ok());
        let account_after_resolve = engine.accounts.get(&client).unwrap();

        // Check that deposit has been reversed and that everything is back to zero
        assert_eq!(
            account_after_resolve.available(),
            account_before_dispute.available()
        );
        assert_eq!(
            account_after_resolve.total(),
            account_before_dispute.total()
        );
        assert_eq!(account_after_resolve.held(), account_before_dispute.held());
        // Resolve should NOT lock account
        assert!(!account_after_resolve.locked());
    }

    #[test]
    fn reject_double_dispute() {
        let mut engine = PaymentEngine::default();

        let client = 1;

        // Deposit
        let deposit = Transaction {
            tx: 1,
            amount: Some(Amount::new(10, 0).unwrap()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        let dispute = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Dispute,
        };
        assert!(engine.insert(dispute).is_ok());

        // Trying to dispute again which should fail
        let dispute = Transaction {
            tx: 1,
            amount: None,
            client,
            disputed: false,
            variant: TransactionVariant::Dispute,
        };
        assert!(engine.insert(dispute).is_err());
    }

    #[test]
    fn reject_unauthenticated_dispute() {
        let mut engine = PaymentEngine::default();

        let client = 1;
        let mallicous_client = 2;

        // Deposit
        let deposit = Transaction {
            tx: 1,
            amount: Some(Amount::new(10, 0).unwrap()),
            client,
            disputed: false,
            variant: TransactionVariant::Deposit,
        };
        assert!(engine.insert(deposit).is_ok());

        // mallicous_client tries to dispute transaction done by another client
        let dispute = Transaction {
            tx: 1,
            amount: None,
            client: mallicous_client,
            disputed: false,
            variant: TransactionVariant::Dispute,
        };
        assert!(engine.insert(dispute).is_err());
    }
}
