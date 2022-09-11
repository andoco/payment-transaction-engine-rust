# Description

Simple transaction processing engine.

# Build, Test, and Run

Build:

```sh
cargo build
```

Test:

````sh
cargo test
`

Run:

```sh
cargo run -- transactions.csv > accounts.csv
````

Run with logging:

```sh
RUST_LOG=info cargo run -- transactions.csv > accounts.csv
```

# Implementation Notes

- The anyhow create is used for faster development but custom error types would be better.
