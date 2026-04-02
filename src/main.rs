mod block;
mod blockchain;
mod transaction;
mod wallet;
mod state;

use crate::blockchain::Blockchain;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use crate::state::State;

fn main() {
    let mut chain = Blockchain::new();
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    chain.state.mint(&alice.address(), 100);

    let tx1 = alice.sign(Transaction {
        from: alice.address(),
        to: bob.address(),
        amount: 10,
        nonce: 0,
        public_key: None,
        signature: None,
    });



    println!("TX valid: {}\n", tx1.verify(&alice.public_key));

    chain.add_block(vec![tx1], &charlie.address()).unwrap();

    for block in &chain.blocks {
        println!("{:?}\n", block);
    }

    println!("Chain valid: {}", chain.is_valid());

    println!("\nEND STATE:\n Alice: {}\n Bob: {}\n Charlie: {}\n", chain.state.get_balance(&alice.address()), chain.state.get_balance(&bob.address()), chain.state.get_balance(&charlie.address()));
}
