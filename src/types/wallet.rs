use k256::ecdsa::{SigningKey, VerifyingKey, signature::Signer};
use rand::rngs::OsRng;
use crate::types::transaction::Transaction;
use sha2::{Sha256, Digest};

pub struct Wallet {
    pub public_key: VerifyingKey,
    pub private_key: SigningKey,
}

impl Wallet {

    // creates a new wallet
    pub fn new() -> Wallet {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        Wallet { public_key, private_key }
    }

    // gets the address from the public key
    pub fn address(&self) -> String {
        let pub_bytes = self.public_key.to_sec1_bytes();
        let hash = Sha256::digest(&pub_bytes);
        hex::encode(&hash[12..])
    }

    // signs a transaction
    pub fn sign(&self, tx: Transaction) -> Transaction {
        let tx_bytes = tx.signable_bytes();
        let sig = self.private_key.sign(&tx_bytes);
        Transaction {
            signature: Some(sig),
            public_key: Some(self.public_key.to_sec1_bytes().to_vec()),
            ..tx
        }
    }

}


// public function to get the address from a public key
pub fn address_from_key(public_key: &VerifyingKey) -> String {
    let pub_bytes = public_key.to_sec1_bytes();
    let hash = Sha256::digest(&pub_bytes);
    hex::encode(&hash[12..])
}