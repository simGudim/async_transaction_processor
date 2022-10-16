#[allow(dead_code)]
mod transaction_processor;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate csv;


use transaction_processor::{
    transaction::{
        Transaction, 
        TransactionState
    },
    processor::Proccessor,
    client_account::ClientAccount
};
use csv::{
    ByteRecord, 
    ReaderBuilder, 
    Trim,
    Writer
};


use std::ffi::OsString;
use std::fs::File;
use std::env;
use std::error::Error;
use std::io;
use std::process;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// returns the positional argument sent to this process. 
// If there are no positional arguments, then this returns an error.
fn get_numbered_arg(pos: usize) -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(pos) {
        None => Err(From::from("expected argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

async fn process_transactions(
    account_map: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    transaction_map: Arc<Mutex<HashMap<u32, TransactionState>>>
) -> Result<(), Box<dyn Error>> {
    // get the input file path 
    let input_file_path = get_numbered_arg(1)?;
    let file = File::open(input_file_path)?;

    // intiliaze reader and allocate memory for the record
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(file);
    let mut raw_record = ByteRecord::new();
    let headers = rdr.byte_headers()?.clone();

    // process transaction
    let mut processor = Proccessor::new();
    while rdr.read_byte_record(&mut raw_record)? {
        let transaction: Transaction = raw_record.deserialize(Some(&headers))?;
        processor.process_transaction(
            transaction, 
            account_map.clone(), 
            transaction_map.clone()
        ).await;
    }

    // output to stdout
    let mut wtr = Writer::from_writer(io::stdout());
    let accounts_map = account_map.lock().unwrap();
    accounts_map
        .iter()
        .try_for_each(|(_, account)| wtr.serialize(account))?;

    wtr.flush()?;
    Ok(())

}

#[tokio::main]
async fn main() {
    let account_map = Arc::new(Mutex::new(HashMap::new()));
    let transaction_map = Arc::new(Mutex::new(HashMap::new()));
    let handle = tokio::spawn(async move  {
        let account_map = account_map.clone();
        let transaction_map = transaction_map.clone();
        match process_transactions(account_map, transaction_map).await {
            Ok(_) => {
                eprintln!("Processing Complete!!! Time to get Shwifty!");
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        }
    });
    handle.await.unwrap();
}