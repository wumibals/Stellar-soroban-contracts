#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, Address, Env, Symbol, symbol_short};

// Import the Policy contract interface to verify ownership and coverage
mod policy_contract {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/policy_contract.wasm");
}

#[contract]
pub struct ClaimsContract;

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const CONFIG: Symbol = symbol_short!("CONFIG");
const CLAIM: Symbol = symbol_short!("CLAIM");
const POLICY_CLAIM: Symbol = symbol_short!("P_CLAIM");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    InsufficientFunds = 4,
    NotFound = 5,
    AlreadyExists = 6,
    InvalidState = 7,
    NotInitialized = 9,
    AlreadyInitialized = 10,
}

fn validate_address(_env: &Env, _address: &Address) -> Result<(), ContractError> {
    Ok(())
}

fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&PAUSED)
        .unwrap_or(false)
}

fn set_paused(env: &Env, paused: bool) {
    env.storage()
        .persistent()
        .set(&PAUSED, &paused);
}

#[contractimpl]
impl ClaimsContract {
    pub fn initialize(env: Env, admin: Address, policy_contract: Address, risk_pool: Address) -> Result<(), ContractError> {
        if env.storage().persistent().has(&ADMIN) {
            return Err(ContractError::AlreadyInitialized);
        }

        validate_address(&env, &admin)?;
        validate_address(&env, &policy_contract)?;
        validate_address(&env, &risk_pool)?;

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&CONFIG, &(policy_contract, risk_pool));
        
        Ok(())
    }

    pub fn submit_claim(env: Env, claimant: Address, policy_id: u64, amount: i128) -> Result<u64, ContractError> {
        // 1. IDENTITY CHECK
        claimant.require_auth();

        if is_paused(&env) {
            return Err(ContractError::Paused);
        }

        // 2. FETCH POLICY DATA
        let (policy_contract_addr, _): (Address, Address) = env.storage()
            .persistent()
            .get(&CONFIG)
            .ok_or(ContractError::NotInitialized)?;

        let policy_client = policy_contract::Client::new(&env, &policy_contract_addr);
        let policy = policy_client.get_policy(&policy_id);

        // 3. OWNERSHIP CHECK (Verify policyholder identity)
        if policy.0 != claimant {
            return Err(ContractError::Unauthorized); 
        }

        // 4. DUPLICATE CHECK (Check if this specific policy already has a claim)
        if env.storage().persistent().has(&(POLICY_CLAIM, policy_id)) {
            return Err(ContractError::AlreadyExists);
        }

        // 5. COVERAGE CHECK (Enforce claim ≤ coverage)
        if amount <= 0 || amount > policy.1 {
            return Err(ContractError::InvalidInput);
        }

        // ID Generation
        let seq: u64 = env.ledger().sequence().into();
        let claim_id = seq + 1; 
        let current_time = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&(CLAIM, claim_id), &(policy_id, claimant.clone(), amount, 0u32, current_time));
        
        env.storage()
            .persistent()
            .set(&(POLICY_CLAIM, policy_id), &claim_id);

        env.events().publish(
            (symbol_short!("clm_sub"), claim_id),
            (policy_id, amount, claimant.clone()),
        );

        Ok(claim_id)
    }

    pub fn get_claim(env: Env, claim_id: u64) -> Result<(u64, Address, i128, u32, u64), ContractError> {
        let claim: (u64, Address, i128, u32, u64) = env
            .storage()
            .persistent()
            .get(&(CLAIM, claim_id))
            .ok_or(ContractError::NotFound)?;
        
        Ok(claim)
    }

    pub fn approve_claim(env: Env, claim_id: u64) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(ContractError::NotInitialized)?;

        admin.require_auth();

        let mut claim: (u64, Address, i128, u32, u64) = env
            .storage()
            .persistent()
            .get(&(CLAIM, claim_id))
            .ok_or(ContractError::NotFound)?;

        if claim.3 != 0u32 && claim.3 != 1u32 {
            return Err(ContractError::InvalidState);
        }

        claim.3 = 2u32;

        env.storage()
            .persistent()
            .set(&(CLAIM, claim_id), &claim);

        env.events().publish(
            (symbol_short!("clm_app"), claim_id),
            (claim.1, claim.2),
        );

        Ok(())
    }

    pub fn pause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(ContractError::NotInitialized)?;

        admin.require_auth();
        set_paused(&env, true);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(ContractError::NotInitialized)?;

        admin.require_auth();
        set_paused(&env, false);
        Ok(())
    }
}
mod test;