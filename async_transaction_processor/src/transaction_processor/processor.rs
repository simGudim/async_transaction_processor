use super::client_account::ClientAccount;
use super::transaction::{
    Transaction, 
    TransactionType, 
    TransactionState
};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;


pub struct Proccessor();


impl Proccessor {
    pub fn new() -> Self {
        Self{}
    }

    // inserts transaction history for deposits and withdrawls
    async fn insert_transaction_history(
        &mut self, 
        transaction: &Transaction,
        transactions_map: Arc<Mutex<HashMap<u32, TransactionState>>>
    ) {
        let mut transactions_map = transactions_map.lock().unwrap();
        transactions_map
            .entry(transaction.tx)
            .or_insert_with(|| {
                TransactionState {
                    client: transaction.client,
                    amount: transaction.amount
                }
            }
        );
    }

    // checks if it is a valid amount within the Option<f64>
    fn check_valid_amount(&self, amount: Option<f64>) -> f64 {
        if let Some(value) = amount {
           value
        } else {
            0.0
        }
    }

    // makes a depist transaction for a particlar account within the HashMap, 
    // otherwise inserts a default account and makes a deposit into the account
    async fn insert_deposit_transaction_into_account(
        &mut self, 
        transaction: &Transaction,
        accounts_map: Arc<Mutex<HashMap<u16, ClientAccount>>>
    ) {
        let amount = self.check_valid_amount(transaction.amount);
        let mut accounts_map = accounts_map.lock().unwrap();
        accounts_map
            .entry(transaction.client)
            .and_modify(|account| {
                account.deposit(amount)
            }).or_insert_with(|| {
                let mut account: ClientAccount = ClientAccount::new(transaction.client);
                account.deposit(amount);
                account
            });
    }

    // makes a depist transaction for a particlar account within the HashMap, 
    // otherwise inserts a default account
    async fn insert_withdrawl_transaction_into_account(
        &mut self, 
        transaction: &Transaction,
        accounts_map: Arc<Mutex<HashMap<u16, ClientAccount>>>
    ) {
        let amount = self.check_valid_amount(transaction.amount);
        let mut accounts_map = accounts_map.lock().unwrap();
        accounts_map
            .entry(transaction.client)
            .and_modify(|account| {
                account.withdrawl(amount)
            }).or_insert_with(|| {
                let account: ClientAccount = ClientAccount::new(transaction.client);
                account
            });
    }

    // handle all the dispute transactions, such as dipiuste, resolve, chargeback
    async fn handle_dipsute_transactions(
        &mut self, 
        transaction: &Transaction,
        accounts_map: Arc<Mutex<HashMap<u16, ClientAccount>>>,
        transactions_map: Arc<Mutex<HashMap<u32, TransactionState>>>
    ) {
        let mut accounts_map = accounts_map.lock().unwrap();
        let mut transactions_map = transactions_map.lock().unwrap();
        let transaction_state = transactions_map.get(&transaction.tx);
        match transaction_state {
            Some(state) => {
                let amount = self.check_valid_amount(state.amount);
                accounts_map
                    .entry(transaction.client)
                    .and_modify(|client| {
                        match transaction.transaction_type {
                            TransactionType::Dispute => client.dispute(amount),
                            // ASSUMPTION: if a transaction is resolved, we no longer need it in memory
                            TransactionType::Resolve => {
                                client.resolve(amount);
                                transactions_map.remove(&transaction.tx);
                            },
                            TransactionType::Chargeback => client.chargeback(amount),
                            _ => eprintln!("something is terribly wrong with your code....")
                        }
                    }
                ).or_insert_with(|| ClientAccount::new(transaction.client));
            },
            None => eprintln!("transaction doesn't exist, maybe something went wrong....")
        }
    }

    // main processor fucntions as an interface for the processing work 
    pub async fn process_transaction(
        &mut self,
        transaction: Transaction,
        accounts_map: Arc<Mutex<HashMap<u16, ClientAccount>>>,
        transactions_map: Arc<Mutex<HashMap<u32, TransactionState>>>
    ) {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                self.insert_transaction_history(&transaction, transactions_map).await;
                self.insert_deposit_transaction_into_account(&transaction, accounts_map).await;
            },
            TransactionType::Withdrawal => {
                self.insert_transaction_history(&transaction, transactions_map).await;
                self.insert_withdrawl_transaction_into_account(&transaction, accounts_map).await;
            },
            _  => self.handle_dipsute_transactions(&transaction, accounts_map, transactions_map).await
        }
    }


    // only used in tests
    pub async fn get_client_account(
        &self, 
        client: u16,
        accounts_map: Arc<Mutex<HashMap<u16, ClientAccount>>>
    ) -> ClientAccount {
        let accounts_map = accounts_map.try_lock().unwrap();
        let value = accounts_map.get(&client).unwrap();
        *value
    }

}
