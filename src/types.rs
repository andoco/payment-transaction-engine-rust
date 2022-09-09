use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: f32,
}

impl Transaction {
    pub fn new(tx_type: &str, client_id: u16, tx_id: u32, amount: f32) -> Self {
        Self {
            tx_type: tx_type.to_string(),
            client_id,
            tx_id,
            amount,
        }
    }
}

#[derive(Default)]
pub struct Account {
    pub client_id: u16,
    pub is_locked: bool,
    pub available_amount: f32,
    pub held_amount: f32,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transaction_sets_fields() {
        let tx = Transaction::new("deposit", 1, 2, 3.0);
        assert_eq!(tx.tx_type, "deposit".to_string());
        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 2);
        assert_eq!(tx.amount, 3.0);
    }

    #[test]
    fn new_account_sets_fields() {
        let acc = Account::new(1);
        assert_eq!(acc.client_id, 1);
        assert_eq!(acc.available_amount, 0.0);
        assert_eq!(acc.held_amount, 0.0);
        assert_eq!(acc.is_locked, false);
    }
}
