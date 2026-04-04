use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::transaction::Transaction;
use serde::{Serialize, Deserialize};
use hex;
use bincode;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn new(index: u64, data: Vec<Transaction>, prev_hash: &str, miner: &str) -> Result<Block, String> {
        let timestamp =SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

        let nonce = 0;
        let hash = Block::compute_hash(index, &data, prev_hash, timestamp, nonce)?;
        Ok(Block {
            index,
            timestamp,
            data: data,
            prev_hash: prev_hash.to_string(),
            hash,
            nonce: nonce,
            miner: miner.to_string(),
        })
    }

    // special function for the genesis block
    pub fn new_genesis(initial_address: &str, initial_supply: u128) -> Block {
        let coinbase = Transaction::coinbase(initial_address, initial_supply);
        let data = vec![coinbase];
        let hash = Block::compute_hash(0, &data, "00000", 0, 0).expect("genesis hash");
        Block {
            index: 0,
            timestamp: 0,
            data,
            prev_hash: "00000".to_string(),
            hash,
            nonce: 0,
            miner: "genesis".to_string(),
        }
    }

    // creates the block hash
    pub fn compute_hash(index: u64, data: &[Transaction], prev_hash: &str, timestamp: u64, nonce: u64) -> Result<String, String> {
        let encoded = bincode::serialize(data).map_err(|e| e.to_string())?;
        let input = format!("{}{}{}{}{}", index, hex::encode(&encoded), prev_hash, timestamp, nonce);
        let result = Sha256::digest(input.as_bytes());
        Ok(format!("{:x}", result))
    }

    // checks previous block is safe
    pub fn prev_block_valid(&self, prev: &Block) -> bool {
        let Ok(hash) = Block::compute_hash(self.index, &self.data, &self.prev_hash, self.timestamp, self.nonce) else {
            return false;
        };
        self.prev_hash == prev.hash && self.hash == hash
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        loop {
            if let Ok(hash) = Block::compute_hash(self.index, &self.data, &self.prev_hash, self.timestamp, self.nonce) {
                self.hash = hash;
                if self.hash.starts_with(&target) {
                    break;
                }
            }
            self.nonce += 1;
        }
    }
}