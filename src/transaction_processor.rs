use crate::account::Account;
use crate::transaction::{Transaction, TransactionType};
use anyhow;
use log::error;
use std::collections::HashMap;
use tokio::sync::mpsc;

// probably not reasonable amounts, this is to ensure no overflow
const MIN_FUNDS: f32 = -1_000_000_000.0;
const MAX_FUNDS: f32 = 1_000_000_000.0;

#[derive(Debug)]
pub struct TransactionProcessor {
    receiver: mpsc::Receiver<anyhow::Result<Transaction>>,
    account_map: HashMap<u16, Account>,
    transaction_map: HashMap<u32, Transaction>,
}

impl TransactionProcessor {
    pub fn new(receiver: mpsc::Receiver<anyhow::Result<Transaction>>) -> TransactionProcessor {
        TransactionProcessor {
            receiver,
            account_map: HashMap::new(),
            transaction_map: HashMap::new(),
        }
    }
    pub fn get_account_map(&self) -> HashMap<u16, Account> {
        self.account_map.clone()
    }
    pub fn display_accounts(&self) {
        println!("client,available,held,total,locked");
        for (_, account) in &self.account_map {
            println!(
                "{},{:.4},{:.4},{:.4},{}",
                account.client_id,
                account.available,
                account.held,
                account.available + account.held,
                account.locked
            )
        }
    }
    pub async fn execute(&mut self) -> &TransactionProcessor {
        while let Some(result_transaction) = self.receiver.recv().await {
            let transaction = match result_transaction {
                Ok(transaction) => transaction,
                Err(e) => {
                    error!("corrupted transaction, {:?}", e);
                    continue;
                }
            };
            if self.account_is_locked(transaction.client_id) {
                self.transaction_can_not_be_performed_error(&transaction, "account is locked");
                continue;
            }
            match transaction.of_type {
                TransactionType::Deposit => self.execute_deposit(transaction),
                TransactionType::Withdrawal => self.execute_withdrawal(transaction),
                TransactionType::Dispute => self.execute_dispute(transaction),
                TransactionType::Resolve => self.execute_resolve(transaction),
                TransactionType::Chargeback => self.execute_chargeback(transaction),
            };
        }
        self
    }
    fn execute_deposit(&mut self, transaction: Transaction) {
        if !self.insert_or_update_account(&transaction, transaction.amount, 0.0, false) {
            return;
        }
        self.insert_transaction(transaction);
    }
    fn execute_withdrawal(&mut self, transaction: Transaction) {
        if !self.account_has_sufficient_funds(&transaction) {
            return;
        }
        if !self.insert_or_update_account(&transaction, -transaction.amount, 0.0, false) {
            return;
        }
        self.insert_transaction(transaction);
    }
    fn execute_dispute(&mut self, transaction: Transaction) {
        if !self.transaction_exists_and_not_under_dispute(&transaction) {
            return;
        }
        let disputed_transaction = self.transaction_map[&transaction.id];
        if !self.transactions_reference_the_same_client(&transaction, &disputed_transaction) {
            return;
        }
        let disputed_amount = self.get_disputed_amount_from_transaction(&disputed_transaction);
        if !self.insert_or_update_account(
            &disputed_transaction,
            -disputed_amount,
            disputed_amount,
            false,
        ) {
            return;
        }
        self.set_transaction_under_dispute(&disputed_transaction, true);
    }
    fn execute_resolve(&mut self, transaction: Transaction) {
        if !self.transaction_exists_and_under_dispute(&transaction) {
            return;
        }
        let disputed_transaction = self.transaction_map[&transaction.id];
        if !self.transactions_reference_the_same_client(&transaction, &disputed_transaction) {
            return;
        }
        let disputed_amount = self.get_disputed_amount_from_transaction(&disputed_transaction);
        if !self.insert_or_update_account(
            &disputed_transaction,
            disputed_amount,
            -disputed_amount,
            false,
        ) {
            return;
        }
        self.set_transaction_under_dispute(&disputed_transaction, false);
    }
    fn execute_chargeback(&mut self, transaction: Transaction) {
        if !self.transaction_exists_and_under_dispute(&transaction) {
            return;
        }
        let disputed_transaction = self.transaction_map[&transaction.id];
        if !self.transactions_reference_the_same_client(&transaction, &disputed_transaction) {
            return;
        }
        let disputed_amount = self.get_disputed_amount_from_transaction(&disputed_transaction);
        if !self.insert_or_update_account(&disputed_transaction, 0.0, -disputed_amount, true) {
            return;
        }
        self.set_transaction_under_dispute(&disputed_transaction, false);
    }

    fn account_is_locked(&mut self, client_id: u16) -> bool {
        match self.account_map.get(&client_id) {
            None => false,
            Some(&account) => account.locked,
        }
    }
    fn account_has_sufficient_funds(&mut self, transaction: &Transaction) -> bool {
        // the account might not be created yet
        self.insert_or_update_account(&transaction, 0.0, 0.0, false);
        if self.account_map[&transaction.client_id].available < transaction.amount {
            self.transaction_can_not_be_performed_error(&transaction, "insufficient funds");
            return false;
        }
        true
    }
    fn insert_or_update_account(
        &mut self,
        transaction: &Transaction,
        available: f32,
        held: f32,
        locked: bool,
    ) -> bool {
        match self.account_map.get_mut(&transaction.client_id) {
            Some(account) => {
                if account.available + available < MIN_FUNDS
                    || account.available + available > MAX_FUNDS
                    || account.held + held < MIN_FUNDS
                    || account.held + held > MAX_FUNDS
                {
                    self.transaction_can_not_be_performed_error(
                        transaction,
                        "your accounts will be out of bounds",
                    );
                    return false;
                }
                account.available += available;
                account.held += held;
                // a bit tricky, but nice and compact :)
                account.locked = account.locked || locked;
                true
            }
            None => {
                self.account_map.insert(
                    transaction.client_id,
                    Account::new(transaction.client_id, available, held, locked),
                );
                true
            }
        }
    }
    fn insert_transaction(&mut self, transaction: Transaction) {
        self.transaction_map.insert(transaction.id, transaction);
    }
    fn set_transaction_under_dispute(&mut self, transaction: &Transaction, under_dispute: bool) {
        match self.transaction_map.get_mut(&transaction.id) {
            Some(transaction) => {
                transaction.under_dispute = under_dispute;
            }
            None => panic!("internal server error"),
        }
    }
    fn transaction_exists_and_not_under_dispute(&self, transaction: &Transaction) -> bool {
        if !(self.transaction_map.contains_key(&transaction.id)
            && !self.transaction_map[&transaction.id].under_dispute)
        {
            self.transaction_can_not_be_performed_error(
                &transaction,
                "transaction does not exist or is already under dispute",
            );
            return false;
        }
        true
    }
    fn transaction_exists_and_under_dispute(&self, transaction: &Transaction) -> bool {
        if !(self.transaction_map.contains_key(&transaction.id)
            && self.transaction_map[&transaction.id].under_dispute)
        {
            self.transaction_can_not_be_performed_error(
                &transaction,
                "transaction does not exist or is not under dispute",
            );
            return false;
        }
        true
    }
    fn get_disputed_amount_from_transaction(&self, transaction: &Transaction) -> f32 {
        // The assumption is amount disputed on Dispute of Withdrawal is negative the
        // amount on transaction. Check README for more details.
        match transaction.of_type {
            TransactionType::Deposit => transaction.amount,
            TransactionType::Withdrawal => -transaction.amount,
            _ => panic!("internal server error"),
        }
    }
    fn transactions_reference_the_same_client(
        &self,
        current_transaction: &Transaction,
        referenced_transaction: &Transaction,
    ) -> bool {
        if current_transaction.client_id != referenced_transaction.client_id {
            self.transaction_can_not_be_performed_error(
                current_transaction,
                "client ids of current and referenced transactions do not match",
            );
            return false;
        }
        true
    }
    fn transaction_can_not_be_performed_error(&self, transaction: &Transaction, message: &str) {
        error!(
            "Transaction: {:?} can not be performed. Reason: {}",
            transaction, message
        );
    }
}
