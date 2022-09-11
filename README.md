# Description

Simple transaction processing engine.

# Build, Test, and Run

Build:

```sh
cargo build
```

Test:

```sh
cargo test
```

Run:

```sh
cargo run -- transactions.csv > accounts.csv
```

Run with logging:

```sh
RUST_LOG=info cargo run -- transactions.csv > accounts.csv
```

Example transaction file:

`transactions.csv`

```
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 5.0
withdrawal, 1, 3, 3.0
```

# Implementation Notes

- The [anyhow](https://docs.rs/anyhow/latest/anyhow/) create is used for faster development but custom error types would be better.
- Transactions are currently stored in `Engine` but ideally they'd be stored using a separate `TransactionManager` and `Engine` would be stateless.
- It is possible to withdraw funds before raising a dispute, which means enough funds might not be available to be held.
  - A withdrawal delay could be implemented to help protect against this and allow more time for disputes.
