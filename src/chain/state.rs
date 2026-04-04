use std::collections::HashMap;
use crate::types::transaction::Transaction;

pub struct State {
    balances: HashMap<String, u128>
}

impl State {

    // creates new State
    pub fn new() -> State {
        State { balances: HashMap::new() }
    }

    // gets a user balances
    pub fn get_balance(&self, address: &str) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    // updates balances for users
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        let sender_balance = self.get_balance(&tx.from);

        if sender_balance < tx.amount {
            return Err("STF".to_string());
        }

        *self.balances.entry(tx.from.clone()).or_insert(0) -= tx.amount;
        *self.balances.entry(tx.to.clone()).or_insert(0) += tx.amount;

        Ok(())
    }

    // updates balance for coinbase tx
    pub fn apply_cb_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        
        if !tx.is_coinbase {
            return Err("not a coinbase transaction".to_string());
        }

        *self.balances.entry(tx.to.clone()).or_insert(0) += tx.amount;

        Ok(())
    }

    pub fn mint(&mut self, address: &str, amount: u128) {
        *self.balances.entry(address.to_string()).or_insert(0) += amount;
    }
}