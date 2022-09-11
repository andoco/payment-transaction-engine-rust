use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Decimal,
}

impl Transaction {
    pub fn new(tx_type: TxType, client_id: u16, tx_id: u32, amount: Decimal) -> Self {
        Self {
            tx_type,
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
    pub available_amount: Decimal,
    pub held_amount: Decimal,
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
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn new_transaction_sets_fields() {
        let tx = Transaction::new(TxType::Deposit, 1, 2, dec!(3.0));
        assert_eq!(tx.tx_type, TxType::Deposit);
        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 2);
        assert_eq!(tx.amount, dec!(3.0));
    }

    #[test]
    fn new_account_sets_fields() {
        let acc = Account::new(1);
        assert_eq!(acc.client_id, 1);
        assert_eq!(acc.available_amount, dec!(0.0));
        assert_eq!(acc.held_amount, dec!(0.0));
        assert_eq!(acc.is_locked, false);
    }
}
