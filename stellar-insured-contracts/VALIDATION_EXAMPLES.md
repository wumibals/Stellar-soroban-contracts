# Validation Examples and Best Practices

This document provides practical examples of how to use the shared validation helpers across contracts.

## Table of Contents

1. [Input Validation Patterns](#input-validation-patterns)
2. [Amount Validation](#amount-validation)
3. [Time Validation](#time-validation)
4. [State Validation](#state-validation)
5. [Arithmetic Operations](#arithmetic-operations)
6. [Batch Validation](#batch-validation)
7. [Error Handling](#error-handling)
8. [Common Patterns](#common-patterns)

---

## Input Validation Patterns

### Basic Address Validation

```rust
use shared::validation::validate_address;
use shared::ContractError;

#[contractimpl]
pub fn create_policy(env: Env, holder: Address) -> Result<u64, ContractError> {
    // Validate holder address
    validate_address(&env, &holder)?;
    
    // Proceed with policy creation
    Ok(1)
}
```

### Multiple Address Validation

```rust
use shared::validation::validate_addresses_different;

#[contractimpl]
pub fn transfer_claim(
    env: Env,
    from: Address,
    to: Address,
) -> Result<(), ContractError> {
    // Ensure different addresses
    validate_addresses_different(&from, &to)?;
    
    // Proceed with transfer
    Ok(())
}
```

---

## Amount Validation

### Single Amount Validation

```rust
use shared::validation::validate_coverage_amount;
use shared::ContractError;

#[contractimpl]
pub fn create_policy(
    env: Env,
    holder: Address,
    coverage_amount: i128,
) -> Result<u64, ContractError> {
    // Validate coverage is within bounds
    validate_coverage_amount(coverage_amount)?;
    
    // Create policy with validated amount
    let policy = Policy {
        coverage_amount,
        // ... other fields
    };
    Ok(policy_id)
}
```

### Premium Validation

```rust
use shared::validation::validate_premium_amount;

#[contractimpl]
pub fn add_premium(
    env: Env,
    policy_id: u64,
    amount: i128,
) -> Result<(), ContractError> {
    // Validate premium amount
    validate_premium_amount(amount)?;
    
    // Apply premium
    Ok(())
}
```

### Custom Amount Bounds

```rust
use shared::validation::validate_amount_in_bounds;

const MINIMUM_STAKE: i128 = 100_000_000;      // 100 units
const MAXIMUM_STAKE: i128 = 10_000_000_000;   // 10k units

#[contractimpl]
pub fn stake(env: Env, amount: i128) -> Result<(), ContractError> {
    // Validate stake amount
    validate_amount_in_bounds(amount, MINIMUM_STAKE, MAXIMUM_STAKE)?;
    
    // Process staking
    Ok(())
}
```

### Funds Availability Check

```rust
use shared::validation::validate_sufficient_funds;

#[contractimpl]
pub fn claim_payout(
    env: Env,
    claim_id: u64,
    amount: i128,
) -> Result<(), ContractError> {
    // Get available balance
    let available_balance = get_pool_balance(&env)?;
    
    // Validate sufficient funds available
    validate_sufficient_funds(available_balance, amount)?;
    
    // Process payout
    Ok(())
}
```

---

## Time Validation

### Future Timestamp Validation

```rust
use shared::validation::validate_future_timestamp;

#[contractimpl]
pub fn create_policy(
    env: Env,
    holder: Address,
    end_time: u64,
) -> Result<u64, ContractError> {
    let current_time = env.ledger().timestamp();
    
    // Validate policy end time is in future
    validate_future_timestamp(current_time, end_time)?;
    
    // Create policy
    Ok(policy_id)
}
```

### Time Range Validation

```rust
use shared::validation::validate_time_range;

#[contractimpl]
pub fn create_proposal(
    env: Env,
    start_time: u64,
    end_time: u64,
) -> Result<u64, ContractError> {
    // Validate start < end
    validate_time_range(start_time, end_time)?;
    
    // Validate both are in future
    let current_time = env.ledger().timestamp();
    validate_future_timestamp(current_time, start_time)?;
    
    // Create proposal
    Ok(proposal_id)
}
```

### Duration Validation

```rust
use shared::validation::validate_duration_days;

#[contractimpl]
pub fn create_policy(
    env: Env,
    duration_days: u32,
) -> Result<u64, ContractError> {
    // Validate duration (1-365 days)
    validate_duration_days(duration_days)?;
    
    // Calculate end time
    let start_time = env.ledger().timestamp();
    let end_time = start_time + (duration_days as u64 * 86_400);
    
    // Create policy
    Ok(policy_id)
}
```

### Claim Grace Period

```rust
use shared::validation::validate_oracle_data_age;
use shared::constants::CLAIM_GRACE_PERIOD_SECONDS;

#[contractimpl]
pub fn submit_claim(
    env: Env,
    policy_id: u64,
    claim_time: u64,
) -> Result<u64, ContractError> {
    let current_time = env.ledger().timestamp();
    
    // Get policy expiry time
    let policy = get_policy(&env, policy_id)?;
    let grace_period_end = policy.end_time + CLAIM_GRACE_PERIOD_SECONDS;
    
    // Validate claim is within grace period
    if claim_time > grace_period_end {
        return Err(ContractError::ClaimPeriodExpired);
    }
    
    // Submit claim
    Ok(claim_id)
}
```

---

## State Validation

### Pause State Check

```rust
use shared::validation::validate_not_paused;

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&DataKey::Paused).unwrap_or(false)
}

#[contractimpl]
pub fn create_policy(env: Env, holder: Address) -> Result<u64, ContractError> {
    // Check contract is not paused
    let paused = is_paused(&env);
    validate_not_paused(paused)?;
    
    // Create policy
    Ok(policy_id)
}
```

### Initialization State Check

```rust
use shared::validation::{validate_initialized, validate_not_initialized};

#[contractimpl]
pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
    // Check not already initialized
    let is_initialized = env.storage()
        .persistent()
        .has(&DataKey::Admin);
    validate_not_initialized(is_initialized)?;
    
    // Set admin
    env.storage().persistent().set(&DataKey::Admin, &admin);
    Ok(())
}

#[contractimpl]
pub fn get_admin(env: Env) -> Result<Address, ContractError> {
    // Check initialized
    let is_initialized = env.storage()
        .persistent()
        .has(&DataKey::Admin);
    validate_initialized(is_initialized)?;
    
    // Get admin
    let admin = env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .ok_or(ContractError::NotInitialized)?;
    Ok(admin)
}
```

---

## Arithmetic Operations

### Safe Addition

```rust
use shared::validation::safe_add;

#[contractimpl]
pub fn deposit(
    env: Env,
    user: Address,
    amount: i128,
) -> Result<(), ContractError> {
    // Get current balance
    let current_balance = get_balance(&env, &user)?;
    
    // Safely add deposit
    let new_balance = safe_add(current_balance, amount)?;
    
    // Update storage
    set_balance(&env, &user, new_balance);
    Ok(())
}
```

### Safe Subtraction

```rust
use shared::validation::safe_sub;

#[contractimpl]
pub fn withdraw(
    env: Env,
    user: Address,
    amount: i128,
) -> Result<(), ContractError> {
    // Get current balance
    let current_balance = get_balance(&env, &user)?;
    
    // Safely subtract withdrawal
    let new_balance = safe_sub(current_balance, amount)?;
    
    // Validate positive balance
    if new_balance < 0 {
        return Err(ContractError::InsufficientFunds);
    }
    
    // Update storage
    set_balance(&env, &user, new_balance);
    Ok(())
}
```

### Safe Multiplication

```rust
use shared::validation::safe_mul;

#[contractimpl]
pub fn calculate_fee(
    env: Env,
    amount: i128,
    fee_percent: u32,
) -> Result<i128, ContractError> {
    // Calculate: amount * fee_percent / 100
    let numerator = safe_mul(amount, fee_percent as i128)?;
    let fee = safe_div(numerator, 100)?;
    Ok(fee)
}
```

### Safe Division

```rust
use shared::validation::safe_div;

#[contractimpl]
pub fn distribute(
    env: Env,
    total: i128,
    recipients_count: u32,
) -> Result<i128, ContractError> {
    // Safely divide total by number of recipients
    let per_recipient = safe_div(total, recipients_count as i128)?;
    Ok(per_recipient)
}
```

---

## Batch Validation

### Multiple Validation Conditions

```rust
use shared::validation::validate_all;
use shared::ContractError;

#[contractimpl]
pub fn create_policy(
    env: Env,
    holder: Address,
    coverage: i128,
    premium: i128,
    duration_days: u32,
) -> Result<u64, ContractError> {
    let current_time = env.ledger().timestamp();
    
    // Validate all inputs at once
    validate_all(&[
        (coverage > 0, ContractError::InvalidInput),
        (coverage <= 1_000_000_000_000_000, ContractError::InvalidCoverageAmount),
        (premium > 0, ContractError::InvalidInput),
        (premium <= 100_000_000_000_000, ContractError::InvalidPremiumAmount),
        (duration_days >= 1, ContractError::InvalidDuration),
        (duration_days <= 365, ContractError::InvalidDuration),
    ])?;
    
    // Proceed with policy creation
    Ok(policy_id)
}
```

---

## Error Handling

### Converting Errors to User Messages

```rust
use shared::ContractError;

fn handle_error(error: ContractError) -> String {
    match error {
        ContractError::InvalidCoverageAmount => {
            "Coverage must be between 1 and 1,000,000 units".to_string()
        }
        ContractError::InsufficientFunds => {
            "Insufficient balance to complete operation".to_string()
        }
        _ => error.message().to_string(),
    }
}
```

### Error Propagation with Context

```rust
use shared::validation::validate_coverage_amount;
use shared::ContractError;

#[contractimpl]
pub fn create_policy(
    env: Env,
    coverage: i128,
) -> Result<u64, ContractError> {
    validate_coverage_amount(coverage)
        .map_err(|_| ContractError::InvalidCoverageAmount)?;
    
    // Create policy
    Ok(policy_id)
}
```

---

## Common Patterns

### Policy Creation Flow

```rust
use shared::validation::*;
use shared::types::PolicyStatus;
use shared::ContractError;

#[contractimpl]
pub fn create_policy(
    env: Env,
    holder: Address,
    coverage_amount: i128,
    premium_amount: i128,
    duration_days: u32,
) -> Result<u64, ContractError> {
    let current_time = env.ledger().timestamp();
    
    // 1. Validate inputs
    validate_address(&env, &holder)?;
    validate_coverage_amount(coverage_amount)?;
    validate_premium_amount(premium_amount)?;
    validate_duration_days(duration_days)?;
    
    // 2. Calculate dates
    let start_time = current_time;
    let end_time = current_time + (duration_days as u64 * 86_400);
    
    // 3. Validate time range
    validate_time_range(start_time, end_time)?;
    
    // 4. Check sufficient funds
    let available = get_available_balance(&env)?;
    validate_sufficient_funds(available, coverage_amount)?;
    
    // 5. Create policy
    let policy = Policy {
        holder,
        coverage_amount,
        premium_amount,
        start_time,
        end_time,
        status: PolicyStatus::Active,
    };
    
    // 6. Store and return
    let policy_id = save_policy(&env, policy)?;
    Ok(policy_id)
}
```

### Claim Processing Flow

```rust
#[contractimpl]
pub fn submit_claim(
    env: Env,
    policy_id: u64,
    amount: i128,
) -> Result<u64, ContractError> {
    let current_time = env.ledger().timestamp();
    
    // 1. Validate amount
    validate_positive_amount(amount)?;
    
    // 2. Get policy
    let policy = get_policy(&env, policy_id)?;
    
    // 3. Validate policy exists and is active
    if policy.status != PolicyStatus::Active {
        return Err(ContractError::InvalidPolicyState);
    }
    
    // 4. Validate claim amount
    if amount > policy.coverage_amount {
        return Err(ContractError::ClaimAmountExceedsCoverage);
    }
    
    // 5. Validate within claim period (with grace period)
    let grace_period_end = policy.end_time + CLAIM_GRACE_PERIOD_SECONDS;
    if current_time > grace_period_end {
        return Err(ContractError::ClaimPeriodExpired);
    }
    
    // 6. Create claim
    let claim = Claim {
        policy_id,
        amount,
        status: ClaimStatus::Submitted,
        submitted_at: current_time,
    };
    
    // 7. Store and return
    let claim_id = save_claim(&env, claim)?;
    Ok(claim_id)
}
```

### Oracle Data Processing

```rust
use shared::validation::validate_oracle_data_age;
use shared::constants::DEFAULT_MAX_ORACLE_DATA_AGE;

#[contractimpl]
pub fn process_oracle_data(
    env: Env,
    data_timestamp: u64,
    value: i128,
) -> Result<(), ContractError> {
    let current_time = env.ledger().timestamp();
    
    // 1. Validate data age
    validate_oracle_data_age(current_time, data_timestamp, DEFAULT_MAX_ORACLE_DATA_AGE)?;
    
    // 2. Validate value is positive
    validate_positive_amount(value)?;
    
    // 3. Get oracle config
    let config = get_oracle_config(&env)?;
    
    // 4. Check submissions
    validate_oracle_submissions(config.min_oracle_submissions)?;
    
    // 5. Store data
    store_oracle_data(&env, value, data_timestamp)?;
    Ok(())
}
```

### Governance Voting

```rust
use shared::validation::validate_percentage;

#[contractimpl]
pub fn create_proposal(
    env: Env,
    voting_period_days: u32,
    min_quorum_percent: u32,
    approval_threshold_percent: u32,
) -> Result<u64, ContractError> {
    // 1. Validate percentages
    validate_percentage(min_quorum_percent)?;
    validate_percentage(approval_threshold_percent)?;
    
    // 2. Validate thresholds make sense
    if approval_threshold_percent <= 50 {
        return Err(ContractError::ThresholdNotMet);
    }
    
    // 3. Calculate voting end time
    let start_time = env.ledger().timestamp();
    let voting_period_seconds = voting_period_days as u64 * 86_400;
    let end_time = start_time + voting_period_seconds;
    
    // 4. Create proposal
    let proposal = Proposal {
        voting_ends_at: end_time,
        min_quorum_percent,
        approval_threshold_percent,
        status: ProposalStatus::Active,
    };
    
    // 5. Store and return
    let proposal_id = save_proposal(&env, proposal)?;
    Ok(proposal_id)
}
```

---

## Best Practices Summary

✅ **DO:**
- Validate all external inputs immediately
- Use safe arithmetic functions for calculations
- Check state before operations
- Validate time ranges explicitly
- Handle all error cases
- Use batch validation for multiple conditions

❌ **DON'T:**
- Trust user input without validation
- Use unchecked arithmetic with untrusted values
- Assume contract state without checking
- Skip time validation on time-dependent operations
- Ignore overflow/underflow possibilities
- Chain validations without error handling

---

## References

- [Shared Validation Module](./shared/src/validation.rs)
- [Shared Error Types](./shared/src/errors.rs)
- [Constants Reference](./shared/src/constants.rs)
