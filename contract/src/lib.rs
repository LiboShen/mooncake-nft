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
use near_contract_standards::non_fungible_token::approval;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenResolver;
use near_contract_standards::non_fungible_token::events::NftMint;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::bs58;
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

mod facai_gen;
mod karma;
mod linkdrop;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    karma: karma::Karma,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    karma: karma::Karma,
    linkdrop: linkdrop::Linkdrops,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml;charset=UTF-8,%3csvg viewBox='0 0 1077 1080' xmlns='http://www.w3.org/2000/svg' xml:space='preserve' fill-rule='evenodd' clip-rule='evenodd' stroke-linecap='round' stroke-linejoin='round' stroke-miterlimit='1.6'%3e%3cpath fill='none' d='M0 0h1076v1079H0z'/%3e%3cpath d='M650 6169v19H544c13-41 75-74 159-81l-1 123-231-1c19-93 134-165 273-165 140 0 256 74 273 169l-231-2s-4-117-3-123c81 8 144 41 155 82l-98-1v-24' fill='none' stroke='gray' stroke-width='27.3' transform='matrix(1.00905 0 0 1.04494 -206 -6074)'/%3e%3cpath d='M650 6169v19H544c13-41 75-74 159-81l-1 123-231-1c19-93 134-165 273-165 140 0 256 74 273 169l-231-2-3-123c81 8 144 41 155 82l-98-1v-24' fill='none' stroke='gray' stroke-width='27.3' transform='matrix(-1.009 .01008 -.01043 -1.04488 1359 6998)'/%3e%3cpath d='M504 363h80a14 14 0 0 0 0-29h-80a14 14 0 0 0 0 29ZM585 569l-79-1a14 14 0 0 0 0 29h79a14 14 0 0 0 0-28ZM735 491v-47a14 14 0 0 0-29 0v47a14 14 0 0 0 29 0ZM385 484v-46a14 14 0 0 0-29-1v47a14 14 0 0 0 29 0Z' fill='gray'/%3e%3cpath d='M474 3843c-13 5-28 7-43 7-42 0-79-20-97-51v-205c18 30 54 51 97 51 15 0 29-3 42-7l1 205ZM858 3539c9-2 17-5 26-10 36-19 57-56 54-93 8-13 13-28 14-45v189a108 108 0 0 1-14 62c3 36-17 73-54 93-9 4-17 7-26 9v-205ZM858 3744c-7 25-25 48-51 61-19 10-39 14-58 12l-1-206a102 102 0 0 0 110-73v206Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-11 -2900)'/%3e%3cpath d='M749 3817c-14 21-37 37-65 43-26 6-52 1-73-11v-205c21 12 47 16 73 11 28-6 50-23 64-44l1 206ZM611 3849c-17 14-40 23-65 23-29 0-54-11-72-29l-1-205c18 17 44 29 73 29 25 0 48-9 65-23v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-11 -2900)'/%3e%3cpath d='M334 3799c-48-2-88-40-93-89v-205c5 48 44 87 93 89v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-12 -2900)'/%3e%3cpath d='M241 3710a101 101 0 0 1-61-97c-7-14-11-29-12-45h0v-1a107 107 0 0 1 0-8v-193c1 15 5 29 12 41v4c0 42 25 78 61 94v205Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-12 -2901)'/%3e%3cpath d='M528 3074c18-15 43-24 69-23 31 1 58 15 76 37 19-4 40-2 60 7 23 10 41 28 52 49a99 99 0 0 1 89 79 98 98 0 0 1 59 106 105 105 0 0 1 0 104c2 36-18 73-55 93-8 4-17 7-26 9-7 25-25 47-51 61-19 10-39 14-58 12-14 21-37 37-65 43-26 6-52 1-73-11-17 14-40 23-65 23-28 0-54-11-72-29-13 5-28 7-43 7-42 0-79-20-97-51-48-2-87-40-93-89a101 101 0 0 1-61-98 103 103 0 0 1 17-121c2-48 36-87 81-96 8-24 27-47 53-61 21-12 43-17 64-15 13-21 35-38 61-45 28-7 56-3 78 9Z' fill='none' stroke='gray' stroke-width='32.9' transform='translate(-7 -2895)'/%3e%3c/svg%3e";
#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    Karma,
    KarmaQuota,
    LinkdropPending,
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
            karma: karma::Karma::new(StorageKey::Karma, StorageKey::KarmaQuota),
            linkdrop: linkdrop::Linkdrops::new(StorageKey::LinkdropPending),
        }
    }

    #[init(ignore_state)]
    pub fn migrate_2022_09_07_linkdrop() -> Self {
        let old_state: OldContract = env::state_read().expect("failed");
        Self {
            tokens: old_state.tokens,
            metadata: old_state.metadata,
            karma: old_state.karma,
            linkdrop: linkdrop::Linkdrops::new(StorageKey::LinkdropPending),
        }
    }

    pub fn reset_karma(&mut self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );
        self.karma = karma::Karma::new(StorageKey::Karma, StorageKey::KarmaQuota);
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
        let token = self.tokens.internal_mint_with_refund(
            token_id,
            receiver_id,
            Some(token_metadata),
            None,
        );
        NftMint {
            owner_id: &token.owner_id,
            token_ids: &[&token.token_id],
            memo: None,
        }
        .emit();
        token
    }

    pub fn top_rank(&self) -> &Vec<(Balance, AccountId)> {
        self.karma.rank()
    }

    #[payable]
    pub fn nft_linkdrop_init(&mut self, token_id: TokenId, pub_key: String) {
        let owner_id = self.tokens.owner_by_id.get(&token_id).unwrap();
        assert_eq!(env::predecessor_account_id(), owner_id, "Unauthorized");
        self.tokens
            .nft_approve(token_id.clone(), env::current_account_id(), None);
        let approval_id: u64 = self
            .tokens
            .approvals_by_id
            .as_ref()
            .unwrap()
            .get(&token_id)
            .unwrap()
            .get(&env::current_account_id())
            .unwrap()
            .clone();
        self.linkdrop
            .add_drop(&pub_key, &token_id, approval_id.clone());
    }

    #[payable]
    pub fn nft_linkdrop_exec(
        &mut self,
        pub_key: String,
        signature: String,
        receiver_id: AccountId,
    ) {
        let linkdrop = self.linkdrop.get_drop(&pub_key, signature);
        self.nft_transfer(
            receiver_id,
            linkdrop.token,
            Some(linkdrop.approval_id),
            None,
        );
        self.linkdrop.remove_drop(&pub_key);
    }
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let sender_id = self.tokens.owner_by_id.get(&token_id).unwrap();
        self.karma.increase(&sender_id, &token_id);
        self.tokens
            .nft_transfer(receiver_id, token_id, approval_id, memo)
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let sender_id = self.tokens.owner_by_id.get(&token_id).unwrap();
        self.karma.increase(&sender_id, &token_id);
        self.tokens
            .nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id)
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
    ) -> bool {
        self.tokens.nft_resolve_transfer(
            previous_owner_id,
            receiver_id,
            token_id,
            approved_account_ids,
        )
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

// near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
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

    use super::*;

    const MINT_COST: u128 = 1000000000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(1).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token = contract.nft_mint_2022(accounts(0));
        assert!(token.token_id.starts_with("2022-"));
        assert_eq!(token.owner_id.to_string(), accounts(0).to_string());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token = contract.nft_mint_2022(accounts(0));

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token.token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token.token_id.clone()) {
            assert_eq!(token.token_id, token.token_id);
            assert_eq!(token.owner_id.to_string(), accounts(1).to_string());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }
}
