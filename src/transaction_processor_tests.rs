#![cfg(test)]
mod tests {
    use crate::account::Account;
    use crate::transaction::{Transaction, TransactionType};
    use crate::transaction_processor::TransactionProcessor;
    use anyhow;
    use maplit::hashmap;
    use tokio::sync::mpsc;

    fn create_transaction_receiver(
        transactions: Vec<Transaction>,
    ) -> mpsc::Receiver<anyhow::Result<Transaction>> {
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            for transaction in transactions {
                sender.send(Ok(transaction)).await.unwrap();
            }
        });
        receiver
    }
    #[tokio::test]
    async fn test_deposit() {
        let transactions: Vec<Transaction> = vec![
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
                client_id: 2,
                amount: 5.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 5.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(
            hashmap! {
                1 => Account::new(1, 15.0, 0.0, false),
                2 => Account::new(2, 5.0, 0.0, false)
            },
            accounts
        );
    }
    #[tokio::test]
    async fn test_withdrawal() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 2,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 5.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 5.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_over_withdrawal() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 2,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 15.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 10.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_dispute() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 0.0, 10.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_dispute_disputed() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 0.0, 10.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_resolve_undisputed() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Resolve,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 10.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_chargeback_undisputed() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Chargeback,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 10.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_resolve_disputed() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Resolve,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 10.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_chargeback_disputed() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Chargeback,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 0.0, 0.0, true)}, accounts);
    }
    #[tokio::test]
    async fn test_transaction_on_frozen_account() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Chargeback,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 2,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 0.0, 0.0, true)}, accounts);
    }
    #[tokio::test]
    async fn test_transaction_on_resolved_account() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 1,
                of_type: TransactionType::Resolve,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 2,
                of_type: TransactionType::Deposit,
                client_id: 1,
                amount: 10.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 20.0, 0.0, false)}, accounts);
    }
    /**
    * example from README
    id 1, deposit 10          			    avail: 10    held: 0   total: 10
    id 2, deposit 100        			    avail: 110   held: 0   total: 110
    id 3, withdraw 50        			    avail: 60    held: 0   total: 60
    dispute id 3 (am_disp = -50)		    avail: 110   held: -50 total: 60
    id 4, withdraw 110                      avail: 0     held: -50 total: -50

    option a) resolve id 3 (am_disp = -50)              avail: -50   held: 0   total: -50  (account not frozen)
    option b) chargeback id 3 (am_disp = -50)           avail: 0     held: 0   total: 0    (account frozen)
    */
    #[tokio::test]
    async fn test_dispute_withdrawal_and_resolve() {
        let transactions: Vec<Transaction> = vec![
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
                amount: 100.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 50.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 4,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 110.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Resolve,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, -50.0, 0.0, false)}, accounts);
    }
    #[tokio::test]
    async fn test_dispute_withdrawal_and_chargeback() {
        let transactions: Vec<Transaction> = vec![
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
                amount: 100.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 50.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
            Transaction {
                id: 4,
                of_type: TransactionType::Withdrawal,
                client_id: 1,
                amount: 110.0,
                under_dispute: false,
            },
            Transaction {
                id: 3,
                of_type: TransactionType::Chargeback,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            },
        ];
        let accounts = TransactionProcessor::new(create_transaction_receiver(transactions))
            .execute()
            .await
            .get_account_map();
        assert_eq!(hashmap! {1 => Account::new(1, 0.0, 0.0, true)}, accounts);
    }
    #[tokio::test]
    async fn test_dispute_non_existent_transactions() {
        assert_eq!(
            hashmap! {},
            TransactionProcessor::new(create_transaction_receiver(vec![Transaction {
                id: 1,
                of_type: TransactionType::Dispute,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            }]))
            .execute()
            .await
            .get_account_map()
        );
        assert_eq!(
            hashmap! {},
            TransactionProcessor::new(create_transaction_receiver(vec![Transaction {
                id: 1,
                of_type: TransactionType::Resolve,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            }]))
            .execute()
            .await
            .get_account_map()
        );
        assert_eq!(
            hashmap! {},
            TransactionProcessor::new(create_transaction_receiver(vec![Transaction {
                id: 1,
                of_type: TransactionType::Chargeback,
                client_id: 1,
                amount: 0.0,
                under_dispute: false,
            }]))
            .execute()
            .await
            .get_account_map()
        );
    }
    #[tokio::test]
    async fn test_out_of_bounds_amounts() {
        assert_eq!(
            hashmap! {1 => Account::new(1, 999_999_999.0, 0.0, false)},
            TransactionProcessor::new(create_transaction_receiver(vec![
                Transaction {
                    id: 1,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 999_999_999.0,
                    under_dispute: false,
                },
                Transaction {
                    id: 1,
                    of_type: TransactionType::Deposit,
                    client_id: 1,
                    amount: 1.01,
                    under_dispute: false,
                }
            ]))
            .execute()
            .await
            .get_account_map()
        );
    }
}
