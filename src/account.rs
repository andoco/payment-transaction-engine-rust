use std::collections::HashMap;

use anyhow::anyhow;
use rust_decimal::Decimal;

use crate::types::Account;

pub trait Manager {
    fn ensure_account(&mut self, client_id: u16) -> anyhow::Result<()>;

    fn deposit(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()>;

    fn withdraw(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()>;

    fn withdraw_held(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()>;

    fn hold(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()>;

    fn release(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()>;

    fn lock(&mut self, client_id: u16) -> anyhow::Result<()>;

    fn is_locked(&mut self, client_id: u16) -> anyhow::Result<bool>;

    fn all(&self) -> Vec<&Account>;
}

pub struct SimpleManager {
    accounts: HashMap<u16, Account>,
}

impl SimpleManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
}

impl Manager for SimpleManager {
    fn ensure_account(&mut self, client_id: u16) -> anyhow::Result<()> {
        if !self.accounts.contains_key(&client_id) {
            self.accounts.insert(client_id, Account::new(client_id));
        }

        Ok(())
    }

    fn deposit(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()> {
        check_positive(amount)?;

        match self.accounts.get_mut(&client_id) {
            Some(acc) => match acc.available_amount.checked_add(amount) {
                Some(new_amount) => {
                    acc.available_amount = new_amount;
                    Ok(())
                }
                None => Err(anyhow!(
                    "Cannot deposit amount as the resulting available amount is too large"
                )),
            },
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn withdraw(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()> {
        check_positive(amount)?;

        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.available_amount - amount < Decimal::ZERO {
                    return Err(anyhow!("Available amount is too low"));
                }

                acc.available_amount -= amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn withdraw_held(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()> {
        check_positive(amount)?;

        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.held_amount - amount < Decimal::ZERO {
                    return Err(anyhow!("Held amount is too low"));
                }

                acc.held_amount -= amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn hold(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()> {
        check_positive(amount)?;

        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.available_amount - amount < Decimal::ZERO {
                    return Err(anyhow!("Available amount is too low"));
                }

                match acc.held_amount.checked_add(amount) {
                    Some(new_amount) => {
                        acc.available_amount -= amount;
                        acc.held_amount = new_amount;
                        Ok(())
                    }
                    None => Err(anyhow!(
                        "Cannot hold amount as the resulting held amount is too large"
                    )),
                }
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn release(&mut self, client_id: u16, amount: Decimal) -> anyhow::Result<()> {
        check_positive(amount)?;

        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.held_amount - amount < Decimal::ZERO {
                    return Err(anyhow!("Held amount is too low"));
                }

                match acc.available_amount.checked_add(amount) {
                    Some(new_amount) => {
                        acc.available_amount = new_amount;
                        acc.held_amount -= amount;
                        Ok(())
                    }
                    None => Err(anyhow!(
                        "Cannot release amount as the resulting available amount is too large"
                    )),
                }
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn lock(&mut self, client_id: u16) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                acc.is_locked = true;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn is_locked(&mut self, client_id: u16) -> anyhow::Result<bool> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => Ok(acc.is_locked),
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn all(&self) -> Vec<&Account> {
        self.accounts.values().collect()
    }
}

fn check_positive(amount: Decimal) -> anyhow::Result<()> {
    match amount.is_sign_positive() {
        true => Ok(()),
        false => Err(anyhow!("The amount is not positive")),
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use crate::types::Transaction;

    use super::*;

    #[test]
    fn check_positive_for_positive_amount_is_ok() {
        assert!(check_positive(dec!(1)).is_ok());
        assert!(check_positive(dec!(0)).is_ok());
    }

    #[test]
    fn check_positive_for_negative_amount_is_err() {
        assert!(check_positive(dec!(-1)).is_err());
    }

    #[test]
    fn ensure_account_creates_account() {
        let mut manager = SimpleManager::new();

        let result = manager.ensure_account(1);
        assert!(result.is_ok());

        let acc = manager.accounts.get(&1).unwrap();
        assert_eq!(acc.client_id, 1);
    }

    #[test]
    fn deposit_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.deposit(1, dec!(10.0));
        assert!(result.is_err());
        assert_eq!(manager.accounts.len(), 0);
    }

    #[test]
    fn deposit_adds_to_available_amount() {
        let mut manager = SimpleManager::new();
        let tx = Transaction::new("desposit", 1, 1, dec!(10.0));

        assert!(manager.ensure_account(tx.client_id).is_ok());

        let result = manager.deposit(tx.client_id, tx.amount);
        assert!(result.is_ok(), "expected ok but got {:?}", result);

        assert_eq!(manager.accounts.len(), 1);

        let acc = manager.accounts.get(&1).expect("Account not found");

        assert_eq!(acc.client_id, tx.client_id);
        assert_eq!(acc.is_locked, false);
        assert_eq!(acc.available_amount, tx.amount);
        assert_eq!(acc.held_amount, dec!(0.0));
    }

    #[test]
    fn deposit_returns_error_when_it_would_cause_overflow() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, Decimal::MAX).is_ok());
        assert!(manager.deposit(client_id, dec!(1.0)).is_err());

        let acc = manager.accounts.get(&1).expect("Account not found");

        assert_eq!(acc.available_amount, Decimal::MAX);
    }

    #[test]
    fn withdraw_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.withdraw(1, dec!(10.0));
        assert!(result.is_err());
    }

    #[test]
    fn withdraw_substracts_from_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.withdraw(client_id, dec!(1.0)).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, dec!(9.0));
    }

    #[test]
    fn withdraw_returns_error_when_amount_greater_than_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.withdraw(client_id, dec!(11.0)).is_err());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, dec!(10.0));
    }

    #[test]
    fn hold_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.hold(1, dec!(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn hold_moves_amount_from_available_amount_to_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.hold(1, dec!(1.0)).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");
        assert_eq!(acc.available_amount, dec!(9.0));
        assert_eq!(acc.held_amount, dec!(1.0));
    }

    #[test]
    fn hold_returns_error_when_amount_greater_than_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;
        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.hold(1, dec!(1.0)).is_err());
    }

    #[test]
    fn hold_returns_error_when_it_would_cause_overflow() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, Decimal::MAX).is_ok());
        assert!(manager.hold(client_id, Decimal::MAX).is_ok());
        assert!(manager.deposit(client_id, dec!(1)).is_ok());
        assert!(manager.hold(client_id, dec!(1)).is_err());

        let acc = manager.accounts.get(&1).expect("Account not found");

        assert_eq!(acc.available_amount, dec!(1));
        assert_eq!(acc.held_amount, Decimal::MAX);
    }

    #[test]
    fn release_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.release(1, dec!(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn release_moves_amount_from_held_amount_to_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.hold(client_id, dec!(1.0)).is_ok());
        assert!(manager.release(client_id, dec!(1.0)).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");
        assert_eq!(acc.available_amount, dec!(10.0));
        assert_eq!(acc.held_amount, dec!(0.0));
    }

    #[test]
    fn release_returns_error_when_amount_greater_than_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;
        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.release(client_id, dec!(1.0)).is_err());
    }

    #[test]
    fn release_returns_error_when_it_would_cause_overflow() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(1)).is_ok());
        assert!(manager.hold(client_id, dec!(1)).is_ok());
        assert!(manager.deposit(client_id, Decimal::MAX).is_ok());
        assert!(manager.release(client_id, dec!(1)).is_err());

        let acc = manager.accounts.get(&1).expect("Account not found");

        assert_eq!(acc.available_amount, Decimal::MAX);
        assert_eq!(acc.held_amount, dec!(1));
    }

    #[test]
    fn withdraw_held_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.withdraw_held(1, dec!(10.0));
        assert!(result.is_err());
    }

    #[test]
    fn withdraw_held_substracts_from_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.hold(client_id, dec!(1.0)).is_ok());
        assert!(manager.withdraw_held(client_id, dec!(1.0)).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, dec!(9.0));
        assert_eq!(acc.held_amount, dec!(0.0));
    }

    #[test]
    fn withdraw_held_returns_error_when_amount_greater_than_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, dec!(10.0)).is_ok());
        assert!(manager.hold(client_id, dec!(1.0)).is_ok());
        assert!(manager.withdraw_held(client_id, dec!(2.0)).is_err());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, dec!(9.0));
        assert_eq!(acc.held_amount, dec!(1.0));
    }

    #[test]
    fn lock_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        assert!(manager.lock(1).is_err());
    }

    #[test]
    fn lock_locks_account() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.lock(client_id).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert!(acc.is_locked);
    }

    #[test]
    fn is_locked_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        assert!(manager.is_locked(1).is_err());
    }

    #[test]
    fn is_locked_returns_false_when_account_is_not_locked() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());

        let result = manager.is_locked(client_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn is_locked_returns_true_when_account_is_locked() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.lock(client_id).is_ok());

        let result = manager.is_locked(client_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
