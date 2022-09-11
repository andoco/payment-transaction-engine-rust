use std::collections::HashMap;

use log::{error, info};

use anyhow::anyhow;

use crate::{
    account,
    types::{Account, Transaction},
};

pub struct Engine<A: account::Manager> {
    accounts: A,
    transactions: HashMap<u32, Transaction>,
}

impl<A: account::Manager> Engine<A> {
    pub fn new(accounts: A) -> Self {
        Self {
            accounts,
            transactions: HashMap::new(),
        }
    }

    fn get_client_tx(&self, client_id: u16, tx_id: u32) -> anyhow::Result<Option<Transaction>> {
        match self.transactions.get(&tx_id) {
            Some(tx) => {
                if tx.client_id == client_id {
                    Ok(Some(tx.clone()))
                } else {
                    Err(anyhow!(
                        "The transaction {} does not belong to client {}",
                        tx_id,
                        client_id
                    ))
                }
            }
            None => Ok(None),
        }
    }

    fn process(&mut self, tx: &Transaction) -> anyhow::Result<()> {
        info!("Ensuring account exists for client id {}", tx.client_id);
        self.accounts.ensure_account(tx.client_id)?;

        if self.accounts.is_locked(tx.client_id)? {
            info!(
                "Account is locked so transaction will not be processed for client id {}",
                tx.client_id
            );
            return Ok(());
        }

        match tx.tx_type.as_str() {
            "deposit" => {
                info!("Depositing amount for client id {}", tx.client_id);
                self.transactions.insert(tx.tx_id, tx.clone());
                self.accounts.deposit(tx.client_id, tx.amount)
            }
            "withdrawal" => {
                info!("Withdrawing amount for client id {}", tx.client_id);
                self.transactions.insert(tx.tx_id, tx.clone());
                self.accounts.withdraw(tx.client_id, tx.amount)
            }
            "dispute" => {
                info!(
                    "Disputing transaction {} for client id {}",
                    tx.tx_id, tx.client_id
                );

                match self.get_client_tx(tx.client_id, tx.tx_id)? {
                    Some(tx) => self.accounts.hold(tx.client_id, tx.amount),
                    None => {
                        info!(
                            "Disputed transaction {} not found so will ignore for client id {}",
                            tx.tx_id, tx.client_id
                        );
                        Ok(())
                    }
                }
            }
            "resolve" => {
                info!(
                    "Resolving transaction {} for client id {}",
                    tx.tx_id, tx.client_id
                );

                match self.get_client_tx(tx.client_id, tx.tx_id)? {
                    Some(held_tx) => self.accounts.release(held_tx.client_id, held_tx.amount),
                    None => {
                        info!(
                            "Resolved transaction {} not found so will ignore for client id {}",
                            tx.tx_id, tx.client_id
                        );
                        Ok(())
                    }
                }
            }
            "chargeback" => {
                info!(
                    "Chargeback transaction {} for client id {}",
                    tx.tx_id, tx.client_id
                );

                match self.get_client_tx(tx.client_id, tx.tx_id)? {
                    Some(tx) => {
                        self.accounts.withdraw_held(tx.client_id, tx.amount)?;
                        self.accounts.lock(tx.client_id)?;
                        Ok(())
                    }
                    None => {
                        info!(
                            "Chargeback transaction {} not found so will ignore for client id {}",
                            tx.tx_id, tx.client_id
                        );
                        Ok(())
                    }
                }
            }
            _ => Err(anyhow!("Unsupported transaction type")),
        }
    }

    pub fn process_all(
        &mut self,
        transactions: impl IntoIterator<Item = anyhow::Result<Transaction>>,
    ) {
        for result in transactions {
            info!("Processing transaction: {:?}", result);

            match result {
                Ok(tx) => match self.process(&tx) {
                    Ok(()) => info!("Transaction complete"),
                    Err(err) => error!("Transaction failed: {}", err),
                },
                Err(err) => error!("Encountered corrupt transaction: {}", err),
            }
        }
    }

    pub fn get_accounts(&self) -> Vec<&Account> {
        self.accounts.all()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn deposit_and_withdrawal_integration_test() {
        let accounts = account::SimpleManager::new();
        let mut engine = Engine::new(accounts);

        let txs = vec![
            Ok(Transaction::new("deposit", 1, 1, dec!(10.0))),
            Ok(Transaction::new("withdrawal", 1, 2, dec!(3.0))),
        ];

        engine.process_all(txs);

        let accounts = engine.get_accounts();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].available_amount, dec!(7.0));
    }

    #[test]
    fn dispute_integration_test() {
        let accounts = account::SimpleManager::new();
        let mut engine = Engine::new(accounts);

        let txs = vec![
            Ok(Transaction::new("deposit", 1, 1, dec!(10.0))),
            Ok(Transaction::new("deposit", 1, 2, dec!(5.0))),
            Ok(Transaction::new("dispute", 1, 1, dec!(0.0))),
        ];

        engine.process_all(txs);

        let accounts = engine.get_accounts();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].available_amount, dec!(5.0));
        assert_eq!(accounts[0].held_amount, dec!(10.0));
    }

    #[test]
    fn chargeback_integration_test() {
        let accounts = account::SimpleManager::new();
        let mut engine = Engine::new(accounts);

        let txs = vec![
            Ok(Transaction::new("deposit", 1, 1, dec!(10.0))),
            Ok(Transaction::new("deposit", 1, 2, dec!(5.0))),
            Ok(Transaction::new("dispute", 1, 1, dec!(0.0))),
            Ok(Transaction::new("chargeback", 1, 1, dec!(0.0))),
            Ok(Transaction::new("withdrawal", 1, 3, dec!(1.0))),
        ];

        engine.process_all(txs);

        let accounts = engine.get_accounts();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(accounts[0].available_amount, dec!(5.0));
        assert_eq!(accounts[0].held_amount, dec!(0.0));
        assert_eq!(accounts[0].is_locked, true);
    }

    #[test]
    fn complex_integration_test() {
        let accounts = account::SimpleManager::new();
        let mut engine = Engine::new(accounts);

        let txs = vec![
            Ok(Transaction::new("deposit", 1, 1, dec!(10.0))),
            Ok(Transaction::new("deposit", 2, 2, dec!(10.0))),
            Ok(Transaction::new("deposit", 1, 3, dec!(5.0))),
            Ok(Transaction::new("dispute", 1, 1, dec!(0.0))),
            Ok(Transaction::new("withdrawal", 2, 4, dec!(3.0))),
            Ok(Transaction::new("chargeback", 1, 1, dec!(0.0))),
        ];

        engine.process_all(txs);

        let accounts = engine.get_accounts();
        assert_eq!(accounts.len(), 2);

        let acc1 = accounts.iter().find(|a| a.client_id == 1).unwrap();
        let acc2 = accounts.iter().find(|a| a.client_id == 2).unwrap();

        assert_eq!(acc1.client_id, 1);
        assert_eq!(acc1.available_amount, dec!(5.0));
        assert_eq!(acc1.held_amount, dec!(0.0));

        assert_eq!(acc2.client_id, 2);
        assert_eq!(acc2.available_amount, dec!(7.0));
        assert_eq!(acc2.held_amount, dec!(0.0));
    }
}
