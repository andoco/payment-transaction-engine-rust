use std::io;

use crate::types::Transaction;
use anyhow::anyhow;
use csv::{Reader, StringRecordsIter};

pub struct CsvTxIter<'a, R: io::Read> {
    reader: CsvTxReader<'a, R>,
}

impl<'a, R: io::Read> Iterator for CsvTxIter<'a, R> {
    type Item = anyhow::Result<Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next()
    }
}

pub struct CsvTxReader<'a, R: io::Read> {
    iter: StringRecordsIter<'a, R>,
}

impl<'a, R: io::Read> CsvTxReader<'a, R> {
    pub fn new(reader: &'a mut Reader<R>) -> Self {
        Self {
            iter: reader.records(),
        }
    }

    fn next(&mut self) -> Option<anyhow::Result<Transaction>> {
        match self.iter.next() {
            Some(Ok(record)) => match record.deserialize::<Transaction>(None) {
                Ok(tx) => Some(Ok(tx)),
                Err(err) => Some(Err(anyhow!(err))),
            },
            Some(Err(err)) => Some(Err(anyhow!(err))),
            None => None,
        }
    }
}

impl<'a, R: io::Read> IntoIterator for CsvTxReader<'a, R> {
    type Item = anyhow::Result<Transaction>;

    type IntoIter = CsvTxIter<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        CsvTxIter { reader: self }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;
    use csv::{ReaderBuilder, Trim};
    use rust_decimal_macros::dec;

    #[test]
    fn iterates_rows() {
        let src =
            "type, client, tx, amount\ndeposit, 1, 1, 1.0\ndeposit, 2, 2, 2.0\nfoo, foo\nfoo, foo, foo, foo";
        let buf = BufReader::new(src.as_bytes());
        let mut csv_reader = ReaderBuilder::new().trim(Trim::All).from_reader(buf);
        let tx_reader = CsvTxReader::new(&mut csv_reader);

        let txs: Vec<_> = tx_reader.into_iter().collect();

        assert_eq!(txs.len(), 4);

        let tx1 = &txs.get(0).unwrap().as_ref().unwrap();
        assert_eq!(tx1.tx_type, "deposit".to_string());
        assert_eq!(tx1.client_id, 1);
        assert_eq!(tx1.tx_id, 1);
        assert_eq!(tx1.amount, dec!(1.0));

        let tx2 = &txs.get(1).unwrap().as_ref().unwrap();
        assert_eq!(tx2.tx_type, "deposit".to_string());
        assert_eq!(tx2.client_id, 2);
        assert_eq!(tx2.tx_id, 2);
        assert_eq!(tx2.amount, dec!(2.0));

        let tx3 = txs.get(2).unwrap();
        assert!(tx3.is_err());

        let tx4 = txs.get(3).unwrap();
        assert!(tx4.is_err());
    }
}
