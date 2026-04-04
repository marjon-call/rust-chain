mod chain;
mod network;
mod types;
mod rpc;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::chain::blockchain::Blockchain;
use crate::chain::genesis::GenesisConfig;
use crate::types::transaction::Transaction;
use crate::types::wallet::Wallet;
use crate::network::node::Node;
use crate::network::node::NodeCommand;
use crate::rpc::server;


#[tokio::main]
async fn main() {
    
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    let genesis = GenesisConfig::load("genesis.json").expect("failed to load genesis.json");
    let mut blockchain = Blockchain::new(&genesis.initial_address, genesis.initial_supply);

    let shared_blockchain = Arc::new(Mutex::new(blockchain));

    // start rpc server
    let rpc_blockchain = Arc::clone(&shared_blockchain);
    tokio::spawn(async move {
        server::start(rpc_blockchain, 8545).await.expect("RPC server failed");
    });
    
    let (mut node, cmd_tx) = Node::new(shared_blockchain).await.expect("failed to create node");

    node.run().await.expect("node failed");
}
