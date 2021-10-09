mod account;
mod amount;
mod engine;
mod error;
mod transaction;

use error::TransactionError;

use std::error::Error;
use std::io;

pub use amount::Amount;
pub use engine::PaymentEngine;
pub use transaction::{Transaction, TransactionVariant};

pub fn run<R: io::Read, W: io::Write>(reader: R, writer: W) -> Result<(), Box<dyn Error>> {
    let mut engine = PaymentEngine::default();

    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        let tx: Transaction = result?;
        if !tx.is_valid() {
            // TODO: maybe stop processing?
            continue;
        }
        match engine.insert(tx) {
            // It is ok to ignore disputes that references a transaction that does not exist
            Err(TransactionError::TransactionNotFound) => (),
            // All other errors should stop the program
            Err(e) => return Err(Box::new(e)),
            _ => (),
        }
    }

    let mut w = csv::Writer::from_writer(writer);
    for client in engine.accounts().values() {
        w.serialize(client)?;
    }

    Ok(())
}
