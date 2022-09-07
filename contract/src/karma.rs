use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, AccountId, Balance, IntoStorageKey};

const RANK_MAX: usize = 10;
const QUOTA_REFILL_SECONDS: u64 = 3600 * 24 * 30;
const MAX_QUOTA: u16 = 3;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Quota {
    value: u16,
    lasted_used_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RankItem {
    account: AccountId,
    balance: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Karma {
    balance_by_owner: LookupMap<AccountId, Balance>,
    rank: Vec<(Balance, AccountId)>,
    quota_by_token_id: LookupMap<TokenId, Quota>,
}

impl Karma {
    pub fn new<Q, R>(balance_by_owner_prefix: Q, quota_by_token_id_prefix: R) -> Self
    where
        Q: IntoStorageKey,
        R: IntoStorageKey,
    {
        Self {
            balance_by_owner: LookupMap::new(balance_by_owner_prefix),
            rank: Vec::new(),
            quota_by_token_id: LookupMap::new(quota_by_token_id_prefix),
        }
    }

    pub fn increase(&mut self, account_id: &AccountId, token_id: &TokenId) {
        let mut quota = match self.quota_by_token_id.get(token_id) {
            Some(quota) => quota,
            None => Quota {
                value: MAX_QUOTA,
                lasted_used_at: env::block_timestamp(),
            },
        };

        if quota.value < MAX_QUOTA {
            let time_delta = env::block_timestamp() - quota.lasted_used_at;
            quota.value = match u64::max(time_delta / QUOTA_REFILL_SECONDS, 0) {
                0 => quota.value,
                1 => u16::max(quota.value + 1, MAX_QUOTA),
                2 => u16::max(quota.value + 2, MAX_QUOTA),
                _ => u16::max(quota.value + 3, MAX_QUOTA),
            };
        }

        // TODO: refill quota if needed
        if quota.value > 0 {
            quota.value -= 1;
            quota.lasted_used_at = env::block_timestamp();
            self.quota_by_token_id.insert(token_id, &quota);
            self.increase_internal(account_id);
        }
    }

    fn increase_internal(&mut self, account_id: &AccountId) {
        let mut balance = self.balance_by_owner.get(account_id).unwrap_or_else(|| 0);
        balance += 1;
        self.balance_by_owner.insert(account_id, &balance);

        for i in 0..self.rank.len() {
            if account_id.eq(&self.rank[i].1) {
                self.rank[i].0 = balance;
                self.rank.sort();
                return;
            }
        }

        if self.rank.len() < RANK_MAX {
            self.rank.push((balance, account_id.clone()));
            self.rank.sort();
            return;
        }

        let rank_min = self.rank.get(0).map_or(0, |item| item.0);
        if balance >= rank_min {
            if balance > rank_min {
                self.rank[0] = (balance, account_id.clone());
            }
            self.rank.sort();
            return;
        }
    }

    pub fn rank(&self) -> &Vec<(Balance, AccountId)> {
        &self.rank
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    fn accounts(id: usize) -> AccountId {
        AccountId::new_unchecked(format!("test_account_{}", id))
    }

    #[test]
    fn test_quota_used_up() {
        let token_id = "test_token_id".to_string();
        let mut karma = Karma::new(b"a", b"b");
        karma.increase(&accounts(0), &token_id);
        assert_eq!(karma.balance_by_owner.get(&accounts(0)), Some(1));
        assert_eq!(
            karma.quota_by_token_id.get(&token_id).map(|q| q.value),
            Some(2)
        );
        assert_eq!(karma.rank.get(0), Some(&(1, accounts(0))));

        karma.increase(&accounts(0), &token_id);
        assert_eq!(karma.balance_by_owner.get(&accounts(0)), Some(2));
        assert_eq!(
            karma.quota_by_token_id.get(&token_id).map(|q| q.value),
            Some(1)
        );
        assert_eq!(karma.rank.get(0), Some(&(1, accounts(0))));

        karma.increase(&accounts(0), &token_id);
        assert_eq!(karma.balance_by_owner.get(&accounts(0)), Some(3));
        assert_eq!(
            karma.quota_by_token_id.get(&token_id).map(|q| q.value),
            Some(0)
        );
        assert_eq!(karma.rank.get(0), Some(&(1, accounts(0))));

        karma.increase(&accounts(0), &token_id);
        assert_eq!(karma.balance_by_owner.get(&accounts(0)), Some(3));
        assert_eq!(
            karma.quota_by_token_id.get(&token_id).map(|q| q.value),
            Some(0)
        );
        assert_eq!(karma.rank.get(0), Some(&(1, accounts(0))));
    }

    #[test]
    fn test_rank_overflow() {
        let mut karma = Karma::new(b"a", b"b");
        for i in 1..12 {
            for j in 0..i {
                karma.increase(&accounts(i), &format!("test_token_{}", i * 10 + j));
            }
        }
        assert_eq!(
            karma.rank,
            vec![
                (2, accounts(2)),
                (3, accounts(3)),
                (4, accounts(4)),
                (5, accounts(5)),
                (6, accounts(6)),
                (7, accounts(7)),
                (8, accounts(8)),
                (9, accounts(9)),
                (10, accounts(10)),
                (11, accounts(11))
            ]
        );
    }

    #[test]
    fn test_rank_update() {
        let mut karma = Karma::new(b"a", b"b");
        karma.increase(&accounts(0), &format!("test_token_{}", 0));
        karma.increase(&accounts(1), &format!("test_token_{}", 1));
        karma.increase(&accounts(0), &format!("test_token_{}", 0));
        karma.increase(&accounts(1), &format!("test_token_{}", 1));
        assert_eq!(karma.rank, vec![(2, accounts(0)), (2, accounts(1)),]);
    }

    #[test]
    fn test_rank_update_2() {
        let mut karma = Karma::new(b"a", b"b");
        karma.increase(&accounts(0), &format!("test_token_{}", 0));
        karma.increase(&accounts(0), &format!("test_token_{}", 0));
        karma.increase(&accounts(0), &format!("test_token_{}", 0));
        karma.increase(&accounts(1), &format!("test_token_{}", 1));
        assert_eq!(karma.rank, vec![(1, accounts(1)), (3, accounts(0)),]);
    }
}
