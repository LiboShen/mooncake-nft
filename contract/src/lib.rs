/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use bs58;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

mod facai_gen;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml;charset=UTF-8,%3csvg viewBox='0 0 1077 1080' xmlns='http://www.w3.org/2000/svg' xml:space='preserve' fill-rule='evenodd' clip-rule='evenodd' stroke-linecap='round' stroke-linejoin='round' stroke-miterlimit='1.6'%3e%3cpath fill='none' d='M0 0h1076v1079H0z'/%3e%3cpath d='M650 6169v19H544c13-41 75-74 159-81l-1 123-231-1c19-93 134-165 273-165 140 0 256 74 273 169l-231-2s-4-117-3-123c81 8 144 41 155 82l-98-1v-24' fill='none' stroke='gray' stroke-width='27.3' transform='matrix(1.00905 0 0 1.04494 -206 -6074)'/%3e%3cpath d='M650 6169v19H544c13-41 75-74 159-81l-1 123-231-1c19-93 134-165 273-165 140 0 256 74 273 169l-231-2-3-123c81 8 144 41 155 82l-98-1v-24' fill='none' stroke='gray' stroke-width='27.3' transform='matrix(-1.009 .01008 -.01043 -1.04488 1359 6998)'/%3e%3cpath d='M504 363h80a14 14 0 0 0 0-29h-80a14 14 0 0 0 0 29ZM585 569l-79-1a14 14 0 0 0 0 29h79a14 14 0 0 0 0-28ZM735 491v-47a14 14 0 0 0-29 0v47a14 14 0 0 0 29 0ZM385 484v-46a14 14 0 0 0-29-1v47a14 14 0 0 0 29 0Z' fill='gray'/%3e%3cpath d='M474 3843c-13 5-28 7-43 7-42 0-79-20-97-51v-205c18 30 54 51 97 51 15 0 29-3 42-7l1 205ZM858 3539c9-2 17-5 26-10 36-19 57-56 54-93 8-13 13-28 14-45v189a108 108 0 0 1-14 62c3 36-17 73-54 93-9 4-17 7-26 9v-205ZM858 3744c-7 25-25 48-51 61-19 10-39 14-58 12l-1-206a102 102 0 0 0 110-73v206Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-11 -2900)'/%3e%3cpath d='M749 3817c-14 21-37 37-65 43-26 6-52 1-73-11v-205c21 12 47 16 73 11 28-6 50-23 64-44l1 206ZM611 3849c-17 14-40 23-65 23-29 0-54-11-72-29l-1-205c18 17 44 29 73 29 25 0 48-9 65-23v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-11 -2900)'/%3e%3cpath d='M334 3799c-48-2-88-40-93-89v-205c5 48 44 87 93 89v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-12 -2900)'/%3e%3cpath d='M241 3710a101 101 0 0 1-61-97c-7-14-11-29-12-45h0v-1a107 107 0 0 1 0-8v-193c1 15 5 29 12 41v4c0 42 25 78 61 94v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-12 -2901)'/%3e%3cpath d='M528 3074c18-15 43-24 69-23 31 1 58 15 76 37 19-4 40-2 60 7 23 10 41 28 52 49a99 99 0 0 1 89 79 98 98 0 0 1 59 106 105 105 0 0 1 0 104c2 36-18 73-55 93-8 4-17 7-26 9-7 25-25 47-51 61-19 10-39 14-58 12-14 21-37 37-65 43-26 6-52 1-73-11-17 14-40 23-65 23-28 0-54-11-72-29-13 5-28 7-43 7-42 0-79-20-97-51-48-2-87-40-93-89a101 101 0 0 1-61-98 103 103 0 0 1 17-121c2-48 36-87 81-96 8-24 27-47 53-61 21-12 43-17 64-15 13-21 35-38 61-45 28-7 56-3 78 9Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-7 -2895)'/%3e%3c/svg%3e";
#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Mooncake".to_string(),
                symbol: "MOONCAKE".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );
        self.tokens
            .internal_mint(token_id, receiver_id, Some(token_metadata))
    }

    #[payable]
    pub fn nft_mint_2022(&mut self, receiver_id: AccountId) -> Token {
        assert!(
            env::attached_deposit() >= 1000000000000000000000000,
            "In sufficient deposit amount"
        );
        let seed = near_sdk::env::random_seed();
        let token_id = format!(
            "2022-{}",
            bs58::encode(&seed)
                .with_alphabet(bs58::Alphabet::BITCOIN)
                .into_string()
        );
        let data_uri = svg_data_uri(crate::facai_gen::new(&seed));
        let token_metadata = TokenMetadata {
            title: Some("恭喜发财".to_string()),
            description: Some("Gong Xi Fa Cai. Mooncake NFT 2022 Edition.".to_string()),
            media: Some(data_uri),
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };
        self.tokens
            .internal_mint(token_id, receiver_id, Some(token_metadata))
    }
}

fn svg_data_uri(svg: String) -> String {
    let encoded = svg
        .replace("%", "%25")
        .replace("> <", "><") // normalise spaces elements
        .replace("; }", ";}") // normalise spaces css
        .replace("<", "%3c")
        .replace(">", "%3e")
        .replace("\"", "'")
        .replace("#", "%23") // needed for ie and firefox
        .replace("{", "%7b")
        .replace("}", "%7d")
        .replace("|", "%7c")
        .replace("^", "%5e")
        .replace("`", "%60")
        .replace("@", "%40");

    format!("{}{}", "data:image/svg+xml;charset=UTF-8,", &encoded)
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
