use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::transaction::Transaction;

#[derive(Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub data: Vec<Transaction>,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub miner: String,
}

impl Block {

    // creates a new block
    pub fn new(index: u64, data: Vec<Transaction>, prev_hash: &str, miner: &str) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data_str = format!("{:?}", data);
        let nonce = 0;
        let hash = Block::compute_hash(index, &data_str, prev_hash, timestamp, nonce);
        Block {
            index,
            timestamp,
            data: data,
            prev_hash: prev_hash.to_string(),
            hash,
            nonce: nonce,
            miner: miner.to_string(),
        }
    }

    // creates the block hash
    pub fn compute_hash(index: u64, data: &str, prev_hash: &str, timestamp: u64, nonce: u64) -> String {
        let input = format!("{}{}{}{}{}", index, data, prev_hash, timestamp, nonce);
        let result = Sha256::digest(input.as_bytes());
        format!("{:x}", result)
    }

    // checks previous block is safe
    pub fn prev_block_valid(&self, prev: &Block) -> bool {
        let data_str = format!("{:?}", self.data);
        self.prev_hash == prev.hash && self.hash == Block::compute_hash(self.index, &data_str, &self.prev_hash, self.timestamp, self.nonce)
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        loop {
            let data_str = format!("{:?}", self.data);
            self.hash = Block::compute_hash(self.index, &data_str, &self.prev_hash, self.timestamp, self.nonce);
            if self.hash.starts_with(&target) {
                break;
            }
            self.nonce += 1;
        }
    }
}