use log::{error, info};

use anyhow::anyhow;

use crate::{account, types::Transaction};

pub struct Engine<A: account::Manager> {
    accounts: A,
}

impl<A: account::Manager> Engine<A> {
    pub fn new(accounts: A) -> Self {
        Self { accounts }
    }

    fn process(&mut self, tx: &Transaction) -> anyhow::Result<()> {
        info!("Ensuring account exists for client id {}", tx.client_id);
        self.accounts
            .ensure_account(tx.client_id)
            .expect("Failed to ensure account exists");

        match tx.tx_type.as_str() {
            "deposit" => {
                info!("Depositing amount for client id {}", tx.client_id);
                self.accounts.deposit(tx.client_id, tx.amount)
            }
            "withdrawal" => {
                info!("Withdrawing amount for client id {}", tx.client_id);
                self.accounts.withdraw(tx.client_id, tx.amount)
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
}
