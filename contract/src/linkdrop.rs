use std::str::FromStr;

use near_contract_standards::non_fungible_token::TokenId;
use near_crypto::{ED25519PublicKey, PublicKey, Signature};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::bs58;
use near_sdk::collections::LookupMap;
use near_sdk::IntoStorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Linkdrop {
    pub token: TokenId,
    pub approval_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Linkdrops {
    pending_drops: LookupMap<String, Linkdrop>,
}

impl Linkdrops {
    pub fn new<Q>(pending_drops_prefix: Q) -> Self
    where
        Q: IntoStorageKey,
    {
        Self {
            pending_drops: LookupMap::new(pending_drops_prefix),
        }
    }

    pub fn add_drop(&mut self, pub_key: &String, token_id: &TokenId, approval_id: u64) {
        self.pending_drops.insert(
            pub_key,
            &Linkdrop {
                token: token_id.clone(),
                approval_id,
            },
        );
    }

    pub fn get_drop(&mut self, pub_key: &String, signature: String) -> Linkdrop {
        assert!(check_priv_key(pub_key, signature));
        self.pending_drops.get(pub_key).unwrap()
    }

    pub fn remove_drop(&mut self, pub_key: &String) {
        self.pending_drops.remove(pub_key);
    }
}

fn check_priv_key(pub_key_str: &String, signature_str: String) -> bool {
    let signature: Signature = signature_str.parse().unwrap();
    let pub_key = PublicKey::from_str(&pub_key_str).unwrap();
    signature.verify(&[0u8; 32], &pub_key)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_linkdrop_init() {}
}
