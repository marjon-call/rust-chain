mod chain;
mod network;
mod types;
mod rpc;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::chain::blockchain::Blockchain;
use crate::chain::genesis::GenesisConfig;
use crate::types::transaction::Transaction;
use crate::types::transaction::TxType;
use crate::types::wallet::Wallet;
use crate::network::node::Node;
use crate::network::node::NodeCommand;
use crate::rpc::server;

// @note for testing purposes here is alices addy and priv key
// addy: f0090076474224898b1ac856772e8e7077845f40
// priv: 854d1faceea7438cd9738802c9d2cfed85a96f2db0a4e2024251fdfd62300198


#[tokio::main]
async fn main() {
    
    let genesis = GenesisConfig::load("genesis.json").expect("failed to load genesis.json");
    let mut blockchain = Blockchain::new(&genesis.initial_address, genesis.initial_supply);

    let shared_blockchain = Arc::new(Mutex::new(blockchain));

    let alice = Wallet::from_private_key("854d1faceea7438cd9738802c9d2cfed85a96f2db0a4e2024251fdfd62300198").unwrap();
    let bob = Wallet::new();

    // PRINT TEST STAKE TX
    let stake_tx = alice.sign(Transaction {
        from: alice.address(),
        to: alice.address(),
        amount: 200,
        nonce: 0,
        public_key: None,
        signature: None,
        is_coinbase: false,
        tx_type: TxType::Stake,
    });

    let stake_bytes = bincode::serialize(&stake_tx).unwrap();
    let stake_hex = hex::encode(&stake_bytes);
    println!("\ncurl STAKE command:");
    println!("curl -X POST http://localhost:8545 \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"method\": \"sendRawTransaction\", \"params\": [\"{}\"], \"id\": 1}}'", stake_hex);


    // PRINT TEST TRANSFER TX
    let tx = alice.sign(Transaction {
        from: alice.address(),
        to: bob.address(),
        amount: 10,
        nonce: 1,
        public_key: None,
        signature: None,
        is_coinbase: false,
        tx_type: TxType::Transfer,
    });

    let bytes = bincode::serialize(&tx).unwrap();
    let hex_tx = hex::encode(&bytes);
    println!("\n\n\nCurl TRANSFER command:");
    println!("curl -X POST http://localhost:8545 \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"method\": \"sendRawTransaction\", \"params\": [\"{}\"], \"id\": 1}}'\n\n", hex_tx);

    // get port
    let rpc_port: u16 = std::env::var("RPC_PORT")
        .unwrap_or("8545".to_string())
        .parse()
        .expect("Invalid RPC_PORT");

    // start rpc server
    let rpc_blockchain = Arc::clone(&shared_blockchain);
    tokio::spawn(async move {
        server::start(rpc_blockchain, rpc_port).await.expect("RPC server failed");
    });

    // @todo dont just pass a random wallet
    let (mut node, cmd_tx) = Node::new(shared_blockchain.clone(), Some(alice)).await.expect("failed to create node");

    let _block_production = Node::start_block_production(Arc::clone(&shared_blockchain), cmd_tx);
    println!("Block production started");
    
    node.run().await.expect("node failed");
}
