use std::collections::HashMap;

use anyhow::anyhow;

use crate::types::Account;

pub trait Manager {
    fn ensure_account(&mut self, client_id: u16) -> anyhow::Result<()>;

    fn deposit(&mut self, client_id: u16, amount: f32) -> anyhow::Result<()>;
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
}
