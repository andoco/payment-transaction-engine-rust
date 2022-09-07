use std::collections::HashMap;

use crate::types::{Account, Transaction};

pub struct Engine {
    accounts: HashMap<u16, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, tx: &Transaction) {
        if !self.accounts.contains_key(&tx.client_id) {
            self.accounts
                .insert(tx.client_id, Account::new(tx.client_id));
        }

        if let Some(account) = self.accounts.get_mut(&tx.client_id) {
            account.available_amount += tx.amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_deposit_for_new_client_creates_account_and_adds_to_available_amount() {
        let tx = Transaction::new("desposit", 1, 1, 10.0);

        let mut engine = Engine::new();

        engine.process(&tx);

        let account = engine
            .accounts
            .get(&tx.client_id)
            .expect("Account not found");

        assert_eq!(account.client_id, tx.client_id);
        assert_eq!(account.is_locked, false);
        assert_eq!(account.available_amount, tx.amount);
        assert_eq!(account.held_amount, 0.0);
    }

    #[test]
    fn process_deposit_for_existing_client_adds_to_available_amount() {
        let mut engine = Engine::new();

        let tx = Transaction::new("desposit", 1, 1, 10.0);
        engine.process(&tx);

        let tx2 = Transaction::new("desposit", 1, 1, 20.0);
        engine.process(&tx2);

        let account = engine
            .accounts
            .get(&tx.client_id)
            .expect("Account not found");

        assert_eq!(account.client_id, tx.client_id);
        assert_eq!(account.is_locked, false);
        assert_eq!(account.available_amount, tx.amount + tx2.amount);
        assert_eq!(account.held_amount, 0.0);
    }
}
