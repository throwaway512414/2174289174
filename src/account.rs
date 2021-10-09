use crate::{amount::Amount, error::TransactionError, TransactionVariant};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Account {
    /// A unique client id
    client: u16,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    /// This should be equal to the `total` - `held` amounts
    available: Amount,
    /// The total funds that are held for dispute.
    /// This should be equal to `total` - `available` amounts
    held: Amount,
    /// The total funds that are available or held.
    /// This should be equal to `available` + `held`
    total: Amount,
    /// Whether the account is locked. An account is locked if a chargeback occurs
    locked: bool,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: Amount::zero(),
            held: Amount::zero(),
            total: Amount::zero(),
            locked: false,
        }
    }

    pub fn available(&self) -> Amount {
        self.available
    }

    pub fn total(&self) -> Amount {
        self.total
    }

    pub fn held(&self) -> Amount {
        self.held
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    fn deposit(&mut self, amount: Amount) {
        self.available += amount;
        self.total += amount;
    }

    fn withdraw(&mut self, amount: Amount) -> Result<(), TransactionError> {
        if self.available < amount {
            return Err(TransactionError::InsufficientFunds);
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    fn dispute(&mut self, amount: Amount) {
        self.available -= amount;
        self.held += amount;
    }

    fn resolve(&mut self, amount: Amount) {
        self.available += amount;
        self.held -= amount;
    }

    fn chargeback(&mut self, amount: Amount) {
        self.total -= amount;
        self.held -= amount;
        self.locked = true;
    }

    pub(crate) fn transaction(
        &mut self,
        variant: &TransactionVariant,
        amount: Amount,
    ) -> Result<(), TransactionError> {
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }

        match variant {
            TransactionVariant::Deposit => {
                self.deposit(amount);
                Ok(())
            }
            TransactionVariant::Withdrawal => self.withdraw(amount),
            TransactionVariant::Dispute => {
                self.dispute(amount);
                Ok(())
            }
            TransactionVariant::Resolve => {
                self.resolve(amount);
                Ok(())
            }
            TransactionVariant::Chargeback => {
                self.chargeback(amount);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chargeback_locks_account() {
        let mut account = Account {
            client: 1,
            available: Amount::new(10, 1).unwrap(),
            total: Amount::new(10, 1).unwrap(),
            held: Amount::zero(),
            locked: false,
        };
        let res = account.transaction(&TransactionVariant::Chargeback, Amount::new(10, 1).unwrap());
        assert!(res.is_ok());
        assert!(account.locked);
    }

    #[test]
    fn locked_account_does_not_permit_any_mutable_operation() {
        let mut account = Account {
            client: 1,
            available: Amount::new(10, 1).unwrap(),
            total: Amount::new(10, 1).unwrap(),
            held: Amount::zero(),
            locked: true,
        };
        let res = account.transaction(&TransactionVariant::Withdrawal, Amount::new(10, 1).unwrap());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), TransactionError::LockedAccount);
    }
}
