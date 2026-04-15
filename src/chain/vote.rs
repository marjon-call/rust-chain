use k256::ecdsa::{VerifyingKey, signature::Verifier, Signature};
use serde::{Serialize, Deserialize};

use crate::types::wallet::address_from_key;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub block_hash: String,
    pub validator_address: String,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
    pub round: u64,
}


impl Vote {

    pub fn signable_bytes(&self) -> Vec<u8> {
        format!("{}{}{}", self.block_hash, self.validator_address, self.round).into_bytes()
    }

    pub fn verify(&self) -> bool {
        match (&self.signature, &self.public_key) {
            (Some(sig_bytes), Some(pk_bytes)) => {
                let Ok(pk) = VerifyingKey::from_sec1_bytes(pk_bytes) else {
                    return false;
                };

                // validate the validator address
                let derived_address = address_from_key(&pk);
                if derived_address != self.validator_address {
                    return false;
                }

                let Ok(sig) = Signature::from_slice(sig_bytes) else {
                    return false;
                };

                pk.verify(&self.signable_bytes(), &sig).is_ok()
            }
            _ => false,
        }
    }

}