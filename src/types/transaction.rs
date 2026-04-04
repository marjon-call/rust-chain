use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};
use serde::{Serialize, Deserialize};
use crate::types::wallet::address_from_key;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub to: String,
    pub from: String,
    pub nonce: u128,
    pub amount: u128,
    pub signature: Option<Signature>,
    pub public_key: Option<Vec<u8>>,
    pub is_coinbase: bool,
}

impl Transaction {
    pub fn verify(&self) -> bool {

        // return early if coinbase
        if self.is_coinbase && self.from == "coinbase" {
            return true;
        }

        match (&self.signature, &self.public_key) {
            (Some(sig), Some(pk_bytes)) => {
                let Ok(pk) = VerifyingKey::from_sec1_bytes(pk_bytes) else {
                    return false;
                };
                let derived_address = address_from_key(&pk);
                if derived_address != self.from {
                    return false;
                }
                let tx_bytes = self.signable_bytes();
                pk.verify(&tx_bytes, sig).is_ok()
            }
            _ => false,
        }
    }

    // formats signable data for a tx
    pub fn signable_bytes(&self) -> Vec<u8> {
        format!("{}{}{}{}", self.from, self.to, self.amount, self.nonce).into_bytes()
    }

    // coinbase tx
    pub fn coinbase(to: &str, amount:u128) -> Transaction {
        Transaction {
            from: "coinbase".to_string(),
            to: to.to_string(),
            amount,
            nonce: 0,
            public_key: None,
            signature: None,
            is_coinbase: true,
        }
    }
}