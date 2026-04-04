use crate::types::transaction::Transaction;

pub struct Mempool {
    pub pending: Vec<Transaction>,
}

impl Mempool {

    pub fn new() -> Mempool {
        Mempool { pending: vec![] }
    }

    pub fn add(&mut self, tx: Transaction) -> Result<(), String> {
        if !tx.verify() {
            return Err("invalid transaction".to_string());
        }
        self.pending.push(tx);
        Ok(())
    }

    pub fn take(&mut self, max: usize) -> Vec<Transaction> {
        let count = max.min(self.pending.len());
        self.pending.drain(..count).collect()
    }



}