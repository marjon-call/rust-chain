use crate::chain::block::Block;
use crate::types::transaction::Transaction;
use crate::chain::state::State;
use crate::chain::mempool::Mempool;

pub const MAX_BLOCK_TXS: usize = 10;

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub state: State,
    pub mempool: Mempool,
}

impl Blockchain {
    // creates new blockchain instance
    pub fn new(initial_address: &str, initial_supply: u128) -> Blockchain {
        let genesis = Block::new_genesis(initial_address, initial_supply);
        println!("Genesis block hash: {}", genesis.hash);
        let mut state = State::new();
        state.apply_cb_transaction(&genesis.data[0]).expect("genesis coinbase failed");
        state.add_validator(initial_address.to_string(), 100).expect("genesis validator failed");
        Blockchain { 
            blocks: vec![genesis],
            state: state,
            mempool: Mempool::new(),
        }
    }

    // adds new block to the chain
    pub fn add_block(&mut self, miner: &str) -> Result<(), String> {
        let txs = self.mempool.take(MAX_BLOCK_TXS);
        let prev = self.blocks.last().ok_or("chain is empty")?;
        let mut block = Block::new(prev.index + 1, txs, &prev.hash, &miner)?;

        // verify txs & apply state changes
        for tx in &block.data {
            // validate
            match &tx.public_key {
                None => return Err("missing public key".to_string()),
                Some(pk) => {
                    if !tx.verify() {
                        return Err("invalid signature".to_string());
                    }
                }
            }

            // update
            self.state.apply_transaction(tx)?;
        }

        // give miner reward
        let coinbase = Transaction::coinbase(miner, 100);
        block.data.push(coinbase.clone());
        self.state.apply_cb_transaction(&coinbase)?;

        // mine the block
        block.mine(2);

        if !block.prev_block_valid(prev) {
            return Err("invalid previous chain state".to_string());
        }

        
        // add block to chain
        self.blocks.push(block);
        Ok(())
    }

    // checks all blocks are valid
    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if !current.prev_block_valid(previous) {
                return false;
            }
        }

        true
    }

    // valiadtes the new block is valid before appending it to our chain
    pub fn validate_and_add(&mut self, new_block: &Block) -> Result<(), String> {
        let curr_block = self.blocks.last().ok_or("chain is empty")?;

        // already have this block, ignore silently
        if new_block.index <= curr_block.index {
            return Ok(()); 
        }

        // checks the new block has valid hash
        if !new_block.prev_block_valid(curr_block) {
            println!("Block validation failed:");
            println!("  new_block.prev_hash: {}", new_block.prev_hash);
            println!("  curr_block.hash: {}", curr_block.hash);
            println!("  new_block.index: {}", new_block.index);
            println!("  curr_block.index: {}", curr_block.index);
            println!("  new_block.hash: {}", new_block.hash);
            println!("  computed hash: {:?}", Block::compute_hash(new_block.index, &new_block.data, &new_block.prev_hash, new_block.timestamp, new_block.nonce));
            return Err("New block was not valid".to_string());
        }

        // validates the tx in the block
        for tx in &new_block.data {
            if tx.is_coinbase {
                self.state.apply_cb_transaction(tx)?;
                continue;
            }
            
            match &tx.public_key {
                None => return Err("missing public key".to_string()),
                Some(_) => {
                    if !tx.verify() {
                        return Err("invalid signature".to_string());
                    }
                }
            }

            // apply tx to state
            if let Err(e) = self.state.apply_state_change(tx) {
                println!("State change failed during sync: {}", e);
            }
        }

        // add block to chain
        self.blocks.push(new_block.clone());
        Ok(())
    }

    // submits tx to the mempool
    pub fn submit_tx(&mut self, tx: Transaction) -> Result<(), String> {
        self.mempool.add(tx)
    }
}