use log::{error, info};

use crate::{account, types::Transaction};

pub struct Engine<A: account::Manager> {
    accounts: A,
}

impl<A: account::Manager> Engine<A> {
    pub fn new(accounts: A) -> Self {
        Self { accounts }
    }

    fn process(&mut self, tx: &Transaction) {
        info!("Ensuring account exists for client id {}", tx.client_id);
        self.accounts
            .ensure_account(tx.client_id)
            .expect("Failed to ensure account exists");

        info!("Depositing amount for client id {}", tx.client_id);
        self.accounts
            .deposit(tx.client_id, tx.amount)
            .expect("Failed to deposit to account");
    }

    pub fn process_all(
        &mut self,
        transactions: impl IntoIterator<Item = anyhow::Result<Transaction>>,
    ) {
        for result in transactions {
            info!("Processing transaction: {:?}", result);

            match result {
                Ok(tx) => {
                    self.process(&tx);
                }
                Err(err) => error!("Encountered corrupt transaction: {}", err),
            }
        }
    }
}
