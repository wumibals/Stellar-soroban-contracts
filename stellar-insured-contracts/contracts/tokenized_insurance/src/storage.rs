use soroban_sdk::{Address, Env};

use crate::errors::Error;
use crate::types::*;

pub fn get_config(env: &Env) -> Result<AdminConfig, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

pub fn set_config(env: &Env, config: &AdminConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_policy_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::PolicyCount)
        .unwrap_or(0)
}

pub fn set_policy_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::PolicyCount, &count);
}

pub fn get_policy(env: &Env, id: u64) -> Result<InsurancePolicy, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Policy(id))
        .ok_or(Error::PolicyNotFound)
}

pub fn set_policy(env: &Env, policy: &InsurancePolicy) {
    env.storage()
        .persistent()
        .set(&DataKey::Policy(policy.id), policy);
}

pub fn get_listing_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::ListingCount)
        .unwrap_or(0)
}

pub fn set_listing_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::ListingCount, &count);
}

pub fn get_listing(env: &Env, id: u64) -> Result<Listing, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Listing(id))
        .ok_or(Error::ListingNotFound)
}

pub fn set_listing(env: &Env, listing: &Listing) {
    env.storage()
        .persistent()
        .set(&DataKey::Listing(listing.id), listing);
}

pub fn get_trigger_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TriggerCount)
        .unwrap_or(0)
}

pub fn set_trigger_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::TriggerCount, &count);
}

pub fn get_trigger(env: &Env, id: u64) -> Result<ParametricTrigger, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Trigger(id))
        .ok_or(Error::TriggerNotFound)
}

pub fn set_trigger(env: &Env, trigger: &ParametricTrigger) {
    env.storage()
        .persistent()
        .set(&DataKey::Trigger(trigger.id), trigger);
}

pub fn get_claim_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::ClaimCount)
        .unwrap_or(0)
}

pub fn set_claim_count(env: &Env, count: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimCount, &count);
}

pub fn get_claim(env: &Env, id: u64) -> Result<Claim, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Claim(id))
        .ok_or(Error::ClaimNotFound)
}

pub fn set_claim(env: &Env, claim: &Claim) {
    env.storage()
        .persistent()
        .set(&DataKey::Claim(claim.id), claim);
}

pub fn get_pool(env: &Env) -> LiquidityPool {
    env.storage()
        .persistent()
        .get(&DataKey::Pool)
        .unwrap_or(LiquidityPool {
            insurance_reserve: 0,
            base_reserve: 0,
            total_lp_tokens: 0,
            fee_rate: 30,
        })
}

pub fn set_pool(env: &Env, pool: &LiquidityPool) {
    env.storage().persistent().set(&DataKey::Pool, pool);
}

pub fn get_balance(env: &Env, owner: &Address, policy_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner.clone(), policy_id))
        .unwrap_or(0)
}

pub fn set_balance(env: &Env, owner: &Address, policy_id: u64, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(owner.clone(), policy_id), &amount);
}

pub fn get_allowance(env: &Env, owner: &Address, spender: &Address, policy_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(
            owner.clone(),
            spender.clone(),
            policy_id,
        ))
        .unwrap_or(0)
}

pub fn set_allowance(
    env: &Env,
    owner: &Address,
    spender: &Address,
    policy_id: u64,
    amount: i128,
) {
    env.storage().persistent().set(
        &DataKey::Allowance(owner.clone(), spender.clone(), policy_id),
        &amount,
    );
}

pub fn get_total_supply(env: &Env, policy_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::TotalSupply(policy_id))
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, policy_id: u64, supply: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::TotalSupply(policy_id), &supply);
}

pub fn get_lp_balance(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::LpBalance(provider.clone()))
        .unwrap_or(0)
}

pub fn set_lp_balance(env: &Env, provider: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::LpBalance(provider.clone()), &amount);
}
