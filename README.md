# Payment Engine

## Quick start

```shell
# Output to stdout
cargo run -- transactions.csv
# Output to file
cargo run -- transactions.csv > accounts.csv
```

## Tests

This will run both unit tests and integration tests

```shell
cargo test
```

The integration tests are located in `tests/e2e.rs` and runs the payment engine
against a set of input files and compares the output to a set of premade output files.

The unit tests are spread around in the `src` directory.

Rust has a strong type system which should be leveraged. 
An example from the code is the `Amount` type which is used to ensure that all amounts 
that are read in from a file is nonnegative and has the correct precision. Also any 
function that depends on its input to have a certain precision can use the `Amount`
type to gurantee correctness.
