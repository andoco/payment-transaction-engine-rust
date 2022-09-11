mod account;
mod engine;
mod reader;
mod types;

use std::env;

use anyhow::anyhow;
use log::info;
use types::Account;

use crate::{engine::Engine, reader::CsvTxReader};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = parse_args(env::args().collect())?;

    info!("Processing transaction file {}", args.transactions_file);

    let file = std::fs::File::open(args.transactions_file)?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let tx_reader = CsvTxReader::new(&mut csv_reader);

    let accounts = account::SimpleManager::new();
    let mut engine = Engine::new(accounts);
    engine.process_all(tx_reader);

    print_accounts(engine.get_accounts());

    Ok(())
}

#[derive(Debug, PartialEq)]
struct Args {
    transactions_file: String,
}

fn parse_args(args: Vec<String>) -> anyhow::Result<Args> {
    match args.get(1) {
        Some(tx_file) => Ok(Args {
            transactions_file: tx_file.to_string(),
        }),
        None => Err(anyhow!("No transaction file provided")),
    }
}

fn print_accounts(accounts: Vec<&Account>) {
    println!("client, available, held, total, locked");
    for acc in accounts {
        println!(
            "{}, {}, {}, {}, {}",
            acc.client_id,
            acc.available_amount.round_dp(4),
            acc.held_amount.round_dp(4),
            (acc.available_amount + acc.held_amount).round_dp(4),
            acc.is_locked
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_should_return_ok() {
        let result = parse_args(vec!["app".to_string(), "transactions.csv".to_string()]);

        assert!(result.is_ok());

        let args = result.unwrap();
        assert_eq!(args.transactions_file, "transactions.csv");
    }

    #[test]
    fn parse_args_should_return_err_when_no_transaction_file_arg() {
        let result = parse_args(vec!["app".to_string()]);

        assert!(result.is_err());

        let err = result.err().unwrap();
        assert_eq!(err.to_string(), "No transaction file provided");
    }
}
