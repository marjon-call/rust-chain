use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};

use crate::types::wallet::address_from_key;

#[derive(Debug)]
pub struct Transaction {
    pub to: String,
    pub from: String,
    pub nonce: u128,
    pub amount: u128,
    pub signature: Option<Signature>,
    pub public_key: Option<VerifyingKey>,
}

impl Transaction {
    pub fn verify(&self, public_key: &VerifyingKey) -> bool {
        match &self.signature {
            None => false,
            Some(sig) => {
                let derived_address = address_from_key(public_key);

                if derived_address != self.from {
                    return false;
                }

                let tx_bytes = self.signable_bytes();
                public_key.verify(&tx_bytes, sig).is_ok()
            }
        }
    }

    // formats signable data for a tx
    pub fn signable_bytes(&self) -> Vec<u8> {
        format!("{}{}{}{}", self.from, self.to, self.amount, self.nonce).into_bytes()
    }
}