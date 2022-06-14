use serde::Deserialize;

pub const MIN_EXCLUSIVE_TRANSACTION_AMOUNT: f32 = 0.0;
// probably not reasonable, but I just want to ensure no overflow
pub const MAX_INCLUSIVE_TRANSACTION_AMOUNT: f32 = 1_000_000_000.0;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct Transaction {
    pub of_type: TransactionType,
    pub client_id: u16,
    pub id: u32,
    pub amount: f32,
    pub under_dispute: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}
