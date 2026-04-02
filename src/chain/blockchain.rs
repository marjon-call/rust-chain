use crate::chain::block::Block;
use crate::types::transaction::Transaction;
use crate::chain::state::State;

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub state: State,
}

impl Blockchain {
    // creates new blockchain instance
    pub fn new() -> Blockchain {
        let genesis = Block::new(0, vec![], "00000", "genesis");
        Blockchain { 
            blocks: vec![genesis],
            state: State::new(),
        }
    }

    // adds new block to the chain
    pub fn add_block(&mut self, data: Vec<Transaction>, miner: &str) -> Result<(), String> {
        let prev = self.blocks.last().ok_or("chain is empty")?;
        let mut block = Block::new(prev.index + 1, data, &prev.hash, &miner);

        // verify txs & apply state changes
        for tx in &block.data {
            // validate
            match &tx.public_key {
                None => return Err("missing public key".to_string()),
                Some(pk) => {
                    if !tx.verify(pk) {
                        return Err("invalid signature".to_string());
                    }
                }
            }

            // update
            self.state.apply_transaction(tx)?;
            self.state.mint(miner, 100);
        }

        // mine the block
        block.mine(2);

        if !block.prev_block_valid(prev) {
            return Err("invalid previous chain state".to_string());
        }

        
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
}