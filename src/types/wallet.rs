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

    // creates a wallet instance from a private key
    pub fn from_private_key(private_key_hex: &str) -> Result<Wallet, String> {
        let bytes = hex::decode(private_key_hex).map_err(|e| e.to_string())?;
        let private_key = SigningKey::from_bytes(bytes.as_slice().into())
            .map_err(|e| e.to_string())?;
        let public_key = VerifyingKey::from(&private_key);
        Ok(Wallet { public_key, private_key })
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