pub mod account;
pub mod transaction;
pub mod transaction_parser;
mod transaction_parser_tests;
pub mod transaction_processor;
mod transaction_processor_tests;

use crate::transaction_parser::TransactionParser;
use crate::transaction_processor::TransactionProcessor;
use std::env;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let input_filename = args[1].clone();

    let channel_size = 1000;
    let (sender, receiver) = mpsc::channel(channel_size);
    tokio::spawn(async move {
        TransactionParser::new(input_filename, sender)
            .parse_transactions()
            .await;
    });

    TransactionProcessor::new(receiver)
        .execute()
        .await
        .display_accounts()
}
