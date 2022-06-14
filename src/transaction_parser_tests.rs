#![cfg(test)]
mod tests {
    use crate::transaction::{Transaction, TransactionType};
    use crate::transaction_parser::TransactionParser;
    use anyhow;
    use tokio::sync::mpsc;
    async fn collect_transactions(
        mut receiver: mpsc::Receiver<anyhow::Result<Transaction>>,
    ) -> Vec<Transaction> {
        let mut transactions = vec![];
        while let Some(transaction) = receiver.recv().await {
            transactions.push(transaction.unwrap())
        }
        transactions
    }
    async fn count_corrupted_transactions(
        mut receiver: mpsc::Receiver<anyhow::Result<Transaction>>,
    ) -> i32 {
        let mut corrupted = 0;
        while let Some(transaction) = receiver.recv().await {
            corrupted += if transaction.is_err() { 1 } else { 0 };
        }
        corrupted
    }
    #[tokio::test]
    async fn test_parse_expected_column_order() {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            TransactionParser::new(
                "test-transactions-expected-column-order.csv".to_string(),
                sender,
            )
            .parse_transactions()
            .await;
        });
        assert_eq!(
            vec![
                Transaction {
                    id: 1,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 10.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 5.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Dispute,
                    client_id: 1,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Resolve,
                    client_id: 1,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 3,
                    of_type: TransactionType::Deposit,
                    client_id: 2,
                    amount: 20.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Withdrawal,
                    client_id: 2,
                    amount: 10.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Dispute,
                    client_id: 2,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Chargeback,
                    client_id: 2,
                    amount: 0.0,
                    under_dispute: false,
                },
            ],
            collect_transactions(receiver).await
        );
    }
    #[tokio::test]
    async fn test_parse_unexpected_column_order() {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            TransactionParser::new(
                "test-transactions-unexpected-column-order.csv".to_string(),
                sender,
            )
            .parse_transactions()
            .await;
        });
        assert_eq!(
            vec![
                Transaction {
                    id: 1,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 10.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 5.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Dispute,
                    client_id: 1,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 2,
                    of_type: TransactionType::Resolve,
                    client_id: 1,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 3,
                    of_type: TransactionType::Deposit,
                    client_id: 2,
                    amount: 20.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Withdrawal,
                    client_id: 2,
                    amount: 10.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Dispute,
                    client_id: 2,
                    amount: 0.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 4,
                    of_type: TransactionType::Chargeback,
                    client_id: 2,
                    amount: 0.0,
                    under_dispute: false,
                },
            ],
            collect_transactions(receiver).await
        );
    }
    #[tokio::test]
    async fn test_corrupted_transactions() {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            TransactionParser::new("test-transactions-corrupted.csv".to_string(), sender)
                .parse_transactions()
                .await;
        });
        assert_eq!(count_corrupted_transactions(receiver).await, 4);
    }
}
