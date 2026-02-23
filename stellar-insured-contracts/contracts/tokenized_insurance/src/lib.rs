#![no_std]

mod amm;
mod errors;
mod ins_token;
mod parametric;
mod storage;
mod trading;
mod types;

use errors::Error;
use soroban_sdk::{contract, contractimpl, token, Address, Env};
use types::*;

#[contract]
pub struct InsuranceContract;

#[contractimpl]
impl InsuranceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        oracle: Address,
        base_token: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        storage::set_config(
            &env,
            &AdminConfig {
                admin,
                oracle,
                base_token,
                paused: false,
            },
        );

        Ok(())
    }

    // ── Policy Management ─────────────────────────────────────────────────────

    pub fn create_policy(
        env: Env,
        policyholder: Address,
        coverage_amount: i128,
        premium: i128,
        policy_type: PolicyType,
        duration_seconds: u64,
        trigger_id: u64,
    ) -> Result<u64, Error> {
        policyholder.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;

        if coverage_amount <= 0 || premium <= 0 {
            return Err(Error::InvalidAmount);
        }

        let base_client = token::Client::new(&env, &config.base_token);
        base_client.transfer(&policyholder, &env.current_contract_address(), &premium);

        let now = env.ledger().timestamp();
        let count = storage::get_policy_count(&env);
        let policy_id = count + 1;

        let policy = InsurancePolicy {
            id: policy_id,
            policyholder: policyholder.clone(),
            coverage_amount,
            premium_paid: premium,
            policy_type,
            start_time: now,
            end_time: now + duration_seconds,
            status: PolicyStatus::Active,
            tokens_minted: coverage_amount,
            trigger_id,
        };

        storage::set_policy(&env, &policy);
        storage::set_policy_count(&env, policy_id);
        ins_token::mint(&env, &policyholder, policy_id, coverage_amount)?;

        Ok(policy_id)
    }

    pub fn expire_policy(env: Env, policy_id: u64) -> Result<(), Error> {
        let mut policy = storage::get_policy(&env, policy_id)?;

        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }

        let now = env.ledger().timestamp();
        if now <= policy.end_time {
            return Err(Error::PolicyNotActive);
        }

        policy.status = PolicyStatus::Expired;
        storage::set_policy(&env, &policy);

        Ok(())
    }

    pub fn get_policy(env: Env, policy_id: u64) -> Result<InsurancePolicy, Error> {
        storage::get_policy(&env, policy_id)
    }

    // ── Insurance Token Standard ──────────────────────────────────────────────

    pub fn balance(env: Env, owner: Address, policy_id: u64) -> i128 {
        storage::get_balance(&env, &owner, policy_id)
    }

    pub fn total_supply(env: Env, policy_id: u64) -> i128 {
        storage::get_total_supply(&env, policy_id)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address, policy_id: u64) -> i128 {
        storage::get_allowance(&env, &owner, &spender, policy_id)
    }

    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        policy_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        owner.require_auth();
        ins_token::approve(&env, &owner, &spender, policy_id, amount)
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        policy_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        from.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        ins_token::transfer(&env, &from, &to, policy_id, amount)
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        policy_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        ins_token::transfer_from(&env, &spender, &from, &to, policy_id, amount)
    }

    pub fn burn(env: Env, from: Address, policy_id: u64, amount: i128) -> Result<(), Error> {
        from.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        let policy = storage::get_policy(&env, policy_id)?;
        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }
        ins_token::burn(&env, &from, policy_id, amount)
    }

    // ── Trading Marketplace ───────────────────────────────────────────────────

    pub fn list_for_sale(
        env: Env,
        seller: Address,
        policy_id: u64,
        amount: i128,
        price_per_token: i128,
    ) -> Result<u64, Error> {
        seller.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        trading::list_for_sale(&env, &seller, policy_id, amount, price_per_token)
    }

    pub fn buy(env: Env, buyer: Address, listing_id: u64, amount: i128) -> Result<(), Error> {
        buyer.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        trading::buy_listing(&env, &buyer, listing_id, amount, &config.base_token)
    }

    pub fn cancel_listing(env: Env, seller: Address, listing_id: u64) -> Result<(), Error> {
        seller.require_auth();
        trading::cancel_listing(&env, &seller, listing_id)
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Result<Listing, Error> {
        storage::get_listing(&env, listing_id)
    }

    // ── AMM Liquidity ─────────────────────────────────────────────────────────

    pub fn add_liquidity(
        env: Env,
        provider: Address,
        policy_id: u64,
        insurance_amount: i128,
        base_amount: i128,
    ) -> Result<i128, Error> {
        provider.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        amm::add_liquidity(
            &env,
            &provider,
            policy_id,
            insurance_amount,
            base_amount,
            &config.base_token,
        )
    }

    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        policy_id: u64,
        lp_amount: i128,
    ) -> Result<(i128, i128), Error> {
        provider.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        amm::remove_liquidity(&env, &provider, policy_id, lp_amount, &config.base_token)
    }

    pub fn swap(
        env: Env,
        user: Address,
        policy_id: u64,
        amount_in: i128,
        min_amount_out: i128,
        insurance_to_base: bool,
    ) -> Result<i128, Error> {
        user.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;
        amm::swap(
            &env,
            &user,
            policy_id,
            amount_in,
            min_amount_out,
            insurance_to_base,
            &config.base_token,
        )
    }

    pub fn get_quote(env: Env, amount_in: i128, insurance_to_base: bool) -> Result<i128, Error> {
        amm::get_quote(&env, amount_in, insurance_to_base)
    }

    pub fn get_pool(env: Env) -> LiquidityPool {
        storage::get_pool(&env)
    }

    pub fn lp_balance(env: Env, provider: Address) -> i128 {
        storage::get_lp_balance(&env, &provider)
    }

    // ── Parametric Triggers ───────────────────────────────────────────────────

    pub fn register_trigger(
        env: Env,
        trigger_type: TriggerType,
        policy_type: PolicyType,
        threshold: i128,
        is_above_threshold: bool,
    ) -> Result<u64, Error> {
        let config = storage::get_config(&env)?;
        config.admin.require_auth();
        parametric::register_trigger(
            &env,
            trigger_type,
            policy_type,
            threshold,
            is_above_threshold,
        )
    }

    pub fn submit_oracle_data(env: Env, trigger_id: u64, value: i128) -> Result<bool, Error> {
        let config = storage::get_config(&env)?;
        config.oracle.require_auth();
        parametric::submit_oracle_data(&env, trigger_id, value)
    }

    pub fn execute_parametric_payout(env: Env, policy_id: u64) -> Result<bool, Error> {
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;

        let triggered = parametric::check_and_payout(&env, policy_id)?;

        if triggered {
            let policy = storage::get_policy(&env, policy_id)?;
            let base_client = token::Client::new(&env, &config.base_token);
            base_client.transfer(
                &env.current_contract_address(),
                &policy.policyholder,
                &policy.coverage_amount,
            );
        }

        Ok(triggered)
    }

    pub fn get_trigger(env: Env, trigger_id: u64) -> Result<ParametricTrigger, Error> {
        storage::get_trigger(&env, trigger_id)
    }

    // ── Claims ────────────────────────────────────────────────────────────────

    pub fn file_claim(
        env: Env,
        policyholder: Address,
        policy_id: u64,
        amount: i128,
    ) -> Result<u64, Error> {
        policyholder.require_auth();
        let config = storage::get_config(&env)?;
        Self::require_active(&config)?;

        let policy = storage::get_policy(&env, policy_id)?;
        if policy.policyholder != policyholder {
            return Err(Error::Unauthorized);
        }
        if policy.status != PolicyStatus::Active {
            return Err(Error::PolicyNotActive);
        }
        if amount <= 0 || amount > policy.coverage_amount {
            return Err(Error::InvalidAmount);
        }

        let now = env.ledger().timestamp();
        if now > policy.end_time {
            return Err(Error::PolicyExpired);
        }

        let count = storage::get_claim_count(&env);
        let claim_id = count + 1;

        let claim = Claim {
            id: claim_id,
            policy_id,
            claimant: policyholder,
            amount,
            is_approved: false,
            is_paid: false,
        };

        storage::set_claim(&env, &claim);
        storage::set_claim_count(&env, claim_id);

        Ok(claim_id)
    }

    pub fn approve_claim(env: Env, claim_id: u64) -> Result<(), Error> {
        let config = storage::get_config(&env)?;
        config.admin.require_auth();

        let mut claim = storage::get_claim(&env, claim_id)?;
        if claim.is_approved || claim.is_paid {
            return Err(Error::ClaimAlreadyProcessed);
        }

        claim.is_approved = true;
        storage::set_claim(&env, &claim);

        Ok(())
    }

    pub fn process_payout(env: Env, claim_id: u64) -> Result<(), Error> {
        let config = storage::get_config(&env)?;
        config.admin.require_auth();

        let mut claim = storage::get_claim(&env, claim_id)?;
        if !claim.is_approved {
            return Err(Error::Unauthorized);
        }
        if claim.is_paid {
            return Err(Error::ClaimAlreadyProcessed);
        }

        let mut policy = storage::get_policy(&env, claim.policy_id)?;
        policy.status = PolicyStatus::Claimed;
        storage::set_policy(&env, &policy);

        let base_client = token::Client::new(&env, &config.base_token);
        base_client.transfer(
            &env.current_contract_address(),
            &claim.claimant,
            &claim.amount,
        );

        claim.is_paid = true;
        storage::set_claim(&env, &claim);

        Ok(())
    }

    pub fn get_claim(env: Env, claim_id: u64) -> Result<Claim, Error> {
        storage::get_claim(&env, claim_id)
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn pause(env: Env) -> Result<(), Error> {
        let mut config = storage::get_config(&env)?;
        config.admin.require_auth();
        config.paused = true;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), Error> {
        let mut config = storage::get_config(&env)?;
        config.admin.require_auth();
        config.paused = false;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn update_oracle(env: Env, new_oracle: Address) -> Result<(), Error> {
        let mut config = storage::get_config(&env)?;
        config.admin.require_auth();
        config.oracle = new_oracle;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let mut config = storage::get_config(&env)?;
        config.admin.require_auth();
        config.admin = new_admin;
        storage::set_config(&env, &config);
        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn require_active(config: &AdminConfig) -> Result<(), Error> {
        if config.paused {
            Err(Error::ContractPaused)
        } else {
            Ok(())
        }
    }
}
