use crate::transaction::{
    Transaction, TransactionType, MAX_INCLUSIVE_TRANSACTION_AMOUNT,
    MIN_EXCLUSIVE_TRANSACTION_AMOUNT,
};
use anyhow::anyhow;
use csv::{ReaderBuilder, Trim};
use log::error;
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Deserialize, Debug)]
struct TransactionDTO {
    #[serde(rename = "type")]
    of_type: TransactionType,
    client: u16,
    tx: u32,
    amount: String,
}

impl TransactionDTO {
    pub fn to_transaction(&self) -> Transaction {
        Transaction {
            of_type: self.of_type,
            client_id: self.client,
            id: self.tx,
            amount: self.amount.parse::<f32>().unwrap_or(0.0),
            under_dispute: false,
        }
    }
}

pub struct TransactionParser {
    path: String,
    sender: mpsc::Sender<anyhow::Result<Transaction>>,
}

impl TransactionParser {
    pub fn new(
        path: String,
        sender: mpsc::Sender<anyhow::Result<Transaction>>,
    ) -> TransactionParser {
        TransactionParser { path, sender }
    }
    pub async fn parse_transactions(&self) {
        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_path(&self.path)
            .unwrap();
        for result in reader.deserialize() {
            let transaction = self.parse_transaction(result);
            if let Err(e) = self.sender.send(transaction).await {
                error!("Failed to send transaction - {:?}", e.to_string());
                return;
            }
        }
    }
    fn parse_transaction(
        &self,
        parser_result: Result<TransactionDTO, csv::Error>,
    ) -> anyhow::Result<Transaction> {
        let transaction_dto: TransactionDTO = parser_result?;
        let transaction = transaction_dto.to_transaction();
        if transaction.of_type == TransactionType::Deposit
            || transaction.of_type == TransactionType::Withdrawal
        {
            if transaction.amount <= MIN_EXCLUSIVE_TRANSACTION_AMOUNT
                || transaction.amount > MAX_INCLUSIVE_TRANSACTION_AMOUNT
            {
                return Err(anyhow!(
                    "Invalid transaction: {}, amount out of bounds",
                    transaction.id
                ));
            }
        }
        Ok(transaction)
    }
}
