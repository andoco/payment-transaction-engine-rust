use std::env;

use anyhow::anyhow;
use log::info;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = parse_args(env::args().collect())?;

    info!("Processing transaction file {}", args.transactions_file);

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
