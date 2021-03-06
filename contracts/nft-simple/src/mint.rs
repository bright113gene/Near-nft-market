use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: Option<TokenId>,
        metadata: TokenMetadata,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
        receiver_id: Option<ValidAccountId>,
        token_type: Option<TokenType>,
    ) {
        let my_token_id = self.token_metadata_by_id.len() + 1;
        if my_token_id >= 2000 {
            return;
        }
        let caller = env::predecessor_account_id();
        let deposit = env::attached_deposit();
        let curr_time = env::block_timestamp() / 1_000_000;

        const PRESALE_TIME: u64 = 1655136000000; // 13th June 2022 04:00PM UTC
        const PUBSALE_TIME: u64 = 1655139600000; // 13th June 2022 05:00PM UTC
        const PRESALE_PRICE: u128 = 30_000_000_000_000_000_000_000_000; // 13th June 2022 05:00PM UTC
        const PUBSALE_PRICE: u128 = 45_000_000_000_000_000_000_000_000; // 13th June 2022 05:00PM UTC

        if curr_time < PRESALE_TIME {
            return;
        } else if curr_time > PRESALE_TIME && curr_time < PUBSALE_TIME {
            if !self.whitelist.contains_key(&caller) || deposit < PRESALE_PRICE {
                return;
            }
        }
        if deposit < PUBSALE_PRICE {
            return;
        }

        let mut final_token_id = format!("{}", my_token_id);
        if let Some(token_id) = token_id {
            final_token_id = token_id;
        }

        let initial_storage_usage = env::storage_usage();
        let mut owner_id = env::predecessor_account_id();
        if let Some(receiver_id) = receiver_id {
            owner_id = receiver_id.into();
        }

        // CUSTOM - create royalty map
        let mut royalty = HashMap::new();
        let mut total_perpetual = 0;
        // user added perpetual_royalties (percentage paid with every transfer)
        if let Some(perpetual_royalties) = perpetual_royalties {
            assert!(
                perpetual_royalties.len() < 7,
                "Cannot add more than 6 perpetual royalty amounts"
            );
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
                total_perpetual += amount;
            }
        }
        // royalty limit for minter capped at 20%
        assert!(
            total_perpetual <= MINTER_ROYALTY_CAP,
            "Perpetual royalties cannot be more than 20%"
        );

        // CUSTOM - enforce minting caps by token_type
        if token_type.is_some() {
            let token_type = token_type.clone().unwrap();
            let cap = u64::from(
                *self
                    .supply_cap_by_type
                    .get(&token_type)
                    .expect("Token type must have supply cap."),
            );
            let supply = u64::from(self.nft_supply_for_type(&token_type));
            assert!(supply < cap, "Cannot mint anymore of token type.");
            let mut tokens_per_type = self.tokens_per_type.get(&token_type).unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::TokensPerTypeInner {
                        token_type_hash: hash_account_id(&token_type),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
            tokens_per_type.insert(&final_token_id);
            self.tokens_per_type.insert(&token_type, &tokens_per_type);
        }
        // END CUSTOM

        let token = Token {
            owner_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
            token_type,
        };
        assert!(
            self.tokens_by_id.insert(&final_token_id, &token).is_none(),
            "Token already exists"
        );
        self.token_metadata_by_id.insert(&final_token_id, &metadata);
        self.internal_add_token_to_owner(&token.owner_id, &final_token_id);

        let new_token_size_in_bytes = env::storage_usage() - initial_storage_usage;
        let required_storage_in_bytes =
            self.extra_storage_in_bytes_per_token + new_token_size_in_bytes;

        refund_deposit(required_storage_in_bytes);
    }

    pub fn add_whitelist(&mut self) {
        self.assert_owner();
        let caller = env::predecessor_account_id();
        self.whitelist.insert(&caller, &(true));
    }

    pub fn remove_whitelist(&mut self) {
        self.assert_owner();
        let caller = env::predecessor_account_id();
        self.whitelist.remove(&caller);
    }

    pub fn get_curr_time(&self) -> u64 {
        return env::block_timestamp() / 1_000_000;
    }

    pub fn is_whitelist(&self, account_id: AccountId) -> bool {
        return self.whitelist.contains_key(&account_id);
    }
}
