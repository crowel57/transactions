mod bank;

use std::{env, error::Error, process};
use crate::bank::{Bank, Transaction};

fn read_transactions(filename: &String) -> Result<(), Box<dyn Error>> {
    let mut bank = Bank::new();
    let mut rdr = csv::ReaderBuilder::new().trim(csv::Trim::All).flexible(true).from_path(filename)?;
    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        bank.insert_txn(transaction);
    }
    println!("{}", bank.to_string());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = args.get(1).unwrap();
        read_file(filename);
    } else {
        println!("No file input parameter");
        process::exit(1);
    }
}

fn read_file(filename: &String) {
    if let Err(err) = read_transactions(filename) {
        println!("error reading transactions: {}", err);
        process::exit(1);
    }
}
