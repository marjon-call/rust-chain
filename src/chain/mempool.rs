use crate::types::transaction::Transaction;

pub struct Mempool {
    pub pending: Vec<Transaction>,
}

impl Mempool {

    pub fn new() -> Mempool {
        Mempool { pending: vec![] }
    }

    pub fn add(&mut self, tx: Transaction) -> Result<(), String> {

        // prevents userds from forging coinbase txs
        if tx.is_coinbase {
            return Err("Mempool: coinbase transactions cannot be submitted externally".to_string());
        }

        if !tx.verify() {
            return Err("Mempool: invalid transaction".to_string());
        }
        self.pending.push(tx);
        Ok(())
    }

    pub fn take(&mut self, max: usize) -> Vec<Transaction> {
        let count = max.min(self.pending.len());
        self.pending.drain(..count).collect()
    }



}