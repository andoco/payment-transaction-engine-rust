use std::collections::HashMap;

use anyhow::anyhow;

use crate::types::Account;

pub trait Manager {
    fn ensure_account(&mut self, client_id: u16) -> anyhow::Result<()>;

    fn deposit(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;

    fn withdraw(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;

    fn withdraw_held(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;

    fn hold(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;

    fn release(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;
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

    fn deposit(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                acc.available_amount += amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn withdraw(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.available_amount - amount < 0.0 {
                    return Err(anyhow!("Available amount is too low"));
                }

                acc.available_amount -= amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn withdraw_held(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.held_amount - amount < 0.0 {
                    return Err(anyhow!("Held amount is too low"));
                }

                acc.held_amount -= amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn hold(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.available_amount - amount < 0.0 {
                    return Err(anyhow!("Available amount is too low"));
                }

                acc.available_amount -= amount;
                acc.held_amount += amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }

    fn release(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()> {
        match self.accounts.get_mut(&client_id) {
            Some(acc) => {
                if acc.held_amount - amount < 0.0 {
                    return Err(anyhow!("Held amount is too low"));
                }

                acc.held_amount -= amount;
                acc.available_amount += amount;
                Ok(())
            }
            None => Err(anyhow!("Account for client {} not found", client_id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Transaction;

    use super::*;

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
        let result = manager.deposit(1, 10.0);
        assert!(result.is_err());
        assert_eq!(manager.accounts.len(), 0);
    }

    #[test]
    fn deposit_adds_to_available_amount() {
        let mut manager = SimpleManager::new();
        let tx = Transaction::new("desposit", 1, 1, 10.0);

        assert!(manager.ensure_account(tx.client_id).is_ok());

        let result = manager.deposit(tx.client_id, tx.amount);
        assert!(result.is_ok(), "expected ok but got {:?}", result);

        assert_eq!(manager.accounts.len(), 1);

        let acc = manager.accounts.get(&1).expect("Account not found");

        assert_eq!(acc.client_id, tx.client_id);
        assert_eq!(acc.is_locked, false);
        assert_eq!(acc.available_amount, tx.amount);
        assert_eq!(acc.held_amount, 0.0);
    }

    #[test]
    fn withdraw_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.withdraw(1, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn withdraw_substracts_from_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.withdraw(client_id, 1.0).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, 9.0);
    }

    #[test]
    fn withdraw_returns_error_when_amount_greater_than_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.withdraw(client_id, 11.0).is_err());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, 10.0);
    }

    #[test]
    fn hold_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.hold(1, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn hold_moves_amount_from_available_amount_to_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.hold(1, 1.0).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");
        assert_eq!(acc.available_amount, 9.0);
        assert_eq!(acc.held_amount, 1.0);
    }

    #[test]
    fn hold_returns_error_when_amount_greater_than_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;
        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.hold(1, 1.0).is_err());
    }

    #[test]
    fn release_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.release(1, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn release_moves_amount_from_held_amount_to_available_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.hold(client_id, 1.0).is_ok());
        assert!(manager.release(client_id, 1.0).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");
        assert_eq!(acc.available_amount, 10.0);
        assert_eq!(acc.held_amount, 0.0);
    }

    #[test]
    fn release_returns_error_when_amount_greater_than_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;
        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.release(client_id, 1.0).is_err());
    }

    #[test]
    fn withdraw_held_returns_error_when_account_not_found() {
        let mut manager = SimpleManager::new();
        let result = manager.withdraw_held(1, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn withdraw_held_substracts_from_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.hold(client_id, 1.0).is_ok());
        assert!(manager.withdraw_held(client_id, 1.0).is_ok());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, 9.0);
        assert_eq!(acc.held_amount, 0.0);
    }

    #[test]
    fn withdraw_held_returns_error_when_amount_greater_than_held_amount() {
        let mut manager = SimpleManager::new();
        let client_id = 1;

        assert!(manager.ensure_account(client_id).is_ok());
        assert!(manager.deposit(client_id, 10.0).is_ok());
        assert!(manager.hold(client_id, 1.0).is_ok());
        assert!(manager.withdraw_held(client_id, 2.0).is_err());

        let acc = manager.accounts.get(&client_id).expect("Account not found");

        assert_eq!(acc.available_amount, 9.0);
        assert_eq!(acc.held_amount, 1.0);
    }
}
