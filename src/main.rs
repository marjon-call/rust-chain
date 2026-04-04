mod chain;
mod network;
mod types;

use crate::chain::blockchain::Blockchain;
use crate::types::transaction::Transaction;
use crate::types::wallet::Wallet;
use crate::network::node::Node;
use crate::network::node::NodeCommand;


#[tokio::main]
async fn main() {
    
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    let mut blockchain = Blockchain::new(&alice.address(), 1000);


    let tx1 = alice.sign(Transaction {
        from: alice.address(),
        to: bob.address(),
        amount: 10,
        nonce: 0,
        public_key: None,
        signature: None,
        is_coinbase: false
    });



    println!("TX valid: {}\n", tx1.verify());

    let (mut node, cmd_tx) = Node::new(blockchain).await.expect("failed to create node");
    let charlie_addr = charlie.address();


    // spawn block production as a seperate task
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        // cmd_tx.send(NodeCommand::SubmitTx(tx1)).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        cmd_tx.send(NodeCommand::AddBlock(charlie.address())).await.unwrap();
    });

    node.run().await.expect("node failed");
}
