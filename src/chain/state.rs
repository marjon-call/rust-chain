use std::collections::HashMap;
use rand::seq::SliceRandom;

use crate::types::transaction::Transaction;
use crate::types::transaction::TxType;
use crate::chain::validator::Validator;

pub const MIN_STAKE: u128 = 100;

pub struct State {
    pub balances: HashMap<String, u128>,
    pub validators: HashMap<String, Validator>,
    pub nonces: HashMap<String, u128>,
}

impl State {

    // creates new State
    pub fn new() -> State {
        State { 
            balances: HashMap::new(),
            validators: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    // gets a user balances
    pub fn get_balance(&self, address: &str) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    // updates balances for users
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        let sender_balance = self.get_balance(&tx.from);

        // check user's balance
        if sender_balance < tx.amount {
            return Err("STF".to_string());
        }

        // check nonce replay
        let last_nonce = self.nonces.get(&tx.from).copied().unwrap_or(0);
        if tx.nonce <= last_nonce && last_nonce != 0 {
            return Err("State: invalid nonce".to_string());
        }

        if let Err(e) = self.apply_state_change(tx) {
            println!("State change failed during sync: {}", e);
        }

        // update nonce
        self.nonces.insert(tx.from.clone(), tx.nonce);

        Ok(())
    }

    // applies state update
    pub fn apply_state_change(&mut self, tx: &Transaction) -> Result<(), String> {
        match tx.tx_type {
            TxType::Transfer => {
                *self.balances.entry(tx.from.clone()).or_insert(0) -= tx.amount;
                *self.balances.entry(tx.to.clone()).or_insert(0) += tx.amount;
            }
            TxType::Stake => {
                *self.balances.entry(tx.from.clone()).or_insert(0) -= tx.amount;
                self.validators.insert(tx.from.clone(), Validator {
                    address: tx.from.clone(),
                    stake: tx.amount,
                    is_active: true,
                    last_proposed: 0,
                });
            }
            TxType::Unstake => {
                if let Some(validator) = self.validators.get_mut(&tx.from) {
                    validator.is_active = false;
                }
                *self.balances.entry(tx.from.clone()).or_insert(0) += tx.amount;
            }
        }
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

    // mints tokens @todo remove 
    pub fn mint(&mut self, address: &str, amount: u128) {
        *self.balances.entry(address.to_string()).or_insert(0) += amount;
    }

    // adds validator to the set
    pub fn add_validator(&mut self, address: String, stake: u128) -> Result<(), String> {
        if stake < MIN_STAKE {
            return Err(format!("State: minimum stake is {}", MIN_STAKE));
        }

        self.validators.insert(address.clone(), Validator {
            address,
            stake,
            is_active: true,
            last_proposed: 0,
        });

        Ok(())
    }

    // gets the active validators
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators.values().filter(|v| v.is_active).collect()
    }

    // chooses validator for next block
    pub fn select_validator(&self, current_block: u64) -> Option<&Validator> {
        let mut active: Vec<&Validator> = self.validators.values()
            .filter(|v| v.is_active && v.last_proposed < current_block)
            .collect();

        if active.is_empty() {
            return None;
        }

        // shuffle first to remove iteration order bias
        active.shuffle(&mut rand::thread_rng());

        // weighted random selection by stake
        let total_stake: u128 = active.iter().map(|v| v.stake).sum();
        let mut rng = rand::random::<u128>() % total_stake;

        for validator in &active {
            if rng < validator.stake {
                return Some(validator);
            }
            rng -= validator.stake;
        }

        active.last().copied()

    }
}