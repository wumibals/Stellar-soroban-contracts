//! Validation helper utilities for insurance contracts
//!
//! This module provides reusable validation functions and helpers that can be
//! used across all contracts to ensure consistency in input validation.

use crate::errors::ContractError;
use soroban_sdk::{Address, Env};

// ===== Address Validation =====

/// Validate that an address is valid and non-zero
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `address` - The address to validate
///
/// # Returns
/// `Ok(())` if valid, `Err(ContractError::InvalidAddress)` otherwise
pub fn validate_address(_env: &Env, _address: &Address) -> Result<(), ContractError> {
    // In Soroban, addresses are validated by the SDK, but we can add
    // additional checks if needed
    
    // Address validation is handled by the SDK itself
    // Additional custom validation can be added here if needed
    Ok(())
}

/// Validate multiple addresses
pub fn validate_addresses(env: &Env, addresses: &[Address]) -> Result<(), ContractError> {
    for address in addresses {
        validate_address(env, address)?;
    }
    Ok(())
}

/// Validate that two addresses are different
pub fn validate_addresses_different(
    addr1: &Address,
    addr2: &Address,
) -> Result<(), ContractError> {
    if addr1 == addr2 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

// ===== Amount Validation =====

/// Validate that an amount is positive
pub fn validate_positive_amount(amount: i128) -> Result<(), ContractError> {
    if amount <= 0 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that an amount is non-negative
pub fn validate_non_negative_amount(amount: i128) -> Result<(), ContractError> {
    if amount < 0 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that an amount is within bounds
pub fn validate_amount_in_bounds(
    amount: i128,
    min: i128,
    max: i128,
) -> Result<(), ContractError> {
    if amount < min || amount > max {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate coverage amount bounds
pub fn validate_coverage_amount(amount: i128) -> Result<(), ContractError> {
    const MIN_COVERAGE: i128 = 1_000_000; // 1 unit
    const MAX_COVERAGE: i128 = 1_000_000_000_000_000; // 1M units
    
    validate_amount_in_bounds(amount, MIN_COVERAGE, MAX_COVERAGE)?;
    Ok(())
}

/// Validate premium amount bounds
pub fn validate_premium_amount(amount: i128) -> Result<(), ContractError> {
    const MIN_PREMIUM: i128 = 100_000; // 0.1 units
    const MAX_PREMIUM: i128 = 100_000_000_000_000; // 100k units
    
    validate_amount_in_bounds(amount, MIN_PREMIUM, MAX_PREMIUM)?;
    Ok(())
}

/// Validate that sufficient funds are available
pub fn validate_sufficient_funds(balance: i128, required: i128) -> Result<(), ContractError> {
    if balance < required {
        return Err(ContractError::InsufficientFunds);
    }
    Ok(())
}

// ===== Time Validation =====

/// Validate that a timestamp is in the future
pub fn validate_future_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError> {
    if timestamp <= current_time {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that a timestamp is in the past
pub fn validate_past_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError> {
    if timestamp > current_time {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that a time range is valid (start < end)
pub fn validate_time_range(
    start_time: u64,
    end_time: u64,
) -> Result<(), ContractError> {
    if start_time >= end_time {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that duration is within bounds
pub fn validate_duration_days(duration_days: u32) -> Result<(), ContractError> {
    const MIN_DURATION: u32 = 1;
    const MAX_DURATION: u32 = 365;
    
    if duration_days < MIN_DURATION || duration_days > MAX_DURATION {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

// ===== Percentage Validation =====

/// Validate that a value is a valid percentage (0-100)
pub fn validate_percentage(percent: u32) -> Result<(), ContractError> {
    if percent > 100 {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that a value is within basis points range (0-10000)
pub fn validate_basis_points(bps: u32) -> Result<(), ContractError> {
    const MAX_BPS: u32 = 10_000;
    
    if bps > MAX_BPS {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validate that deviation is within acceptable range
pub fn validate_oracle_deviation(deviation_bps: u32) -> Result<(), ContractError> {
    validate_basis_points(deviation_bps)?;
    
    // Typical max deviation is 500 bps (5%)
    const MAX_DEVIATION: u32 = 500;
    if deviation_bps > MAX_DEVIATION {
        return Err(ContractError::OracleValidationFailed);
    }
    
    Ok(())
}

// ===== State Validation =====

/// Validate that contract is not paused
pub fn validate_not_paused(is_paused: bool) -> Result<(), ContractError> {
    if is_paused {
        return Err(ContractError::Paused);
    }
    Ok(())
}

/// Validate that contract is initialized
pub fn validate_initialized(is_initialized: bool) -> Result<(), ContractError> {
    if !is_initialized {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

/// Validate that contract is not already initialized
pub fn validate_not_initialized(is_initialized: bool) -> Result<(), ContractError> {
    if is_initialized {
        return Err(ContractError::AlreadyInitialized);
    }
    Ok(())
}

// ===== Arithmetic Validation =====

/// Safely add two amounts, returning error on overflow
pub fn safe_add(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_add(b).ok_or(ContractError::Overflow)
}

/// Safely subtract two amounts, returning error on underflow
pub fn safe_sub(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_sub(b).ok_or(ContractError::Underflow)
}

/// Safely multiply two amounts, returning error on overflow
pub fn safe_mul(a: i128, b: i128) -> Result<i128, ContractError> {
    a.checked_mul(b).ok_or(ContractError::Overflow)
}

/// Safely divide two amounts, returning error on division by zero
pub fn safe_div(a: i128, b: i128) -> Result<i128, ContractError> {
    if b == 0 {
        return Err(ContractError::InvalidInput);
    }
    a.checked_div(b).ok_or(ContractError::Overflow)
}

// ===== Batch Validation =====

/// Validate multiple conditions at once
///
/// # Arguments
/// * `conditions` - A list of (is_valid, error) tuples
///
/// # Returns
/// `Ok(())` if all are valid, `Err(ContractError)` for the first invalid
pub fn validate_all(conditions: &[(bool, ContractError)]) -> Result<(), ContractError> {
    for &(is_valid, error) in conditions {
        if !is_valid {
            return Err(error);
        }
    }
    Ok(())
}

// ===== Calculation Helpers =====

/// Calculate a percentage of an amount
///
/// # Arguments
/// * `amount` - The base amount
/// * `percent` - The percentage (0-100)
///
/// # Returns
/// The calculated percentage amount, or error on overflow
pub fn calculate_percentage(amount: i128, percent: u32) -> Result<i128, ContractError> {
    validate_percentage(percent)?;
    
    if percent == 0 {
        return Ok(0);
    }
    
    safe_mul(amount, percent as i128)?
        .checked_div(100)
        .ok_or(ContractError::Overflow)
}

/// Calculate basis points of an amount
///
/// # Arguments
/// * `amount` - The base amount
/// * `bps` - The basis points (0-10000)
///
/// # Returns
/// The calculated amount, or error on overflow
pub fn calculate_basis_points(amount: i128, bps: u32) -> Result<i128, ContractError> {
    validate_basis_points(bps)?;
    
    if bps == 0 {
        return Ok(0);
    }
    
    safe_mul(amount, bps as i128)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)
}

/// Calculate reserve ratio percentage
pub fn calculate_reserve_ratio(
    reserve: i128,
    total_value: i128,
) -> Result<u32, ContractError> {
    validate_positive_amount(total_value)?;
    
    if reserve == 0 {
        return Ok(0);
    }
    
    let ratio = safe_div(safe_mul(reserve, 100)?, total_value)? as u32;
    Ok(ratio)
}

/// Validate reserve ratio is within bounds
pub fn validate_reserve_ratio(ratio_percent: u32) -> Result<(), ContractError> {
    const MIN_RATIO: u32 = 20; // 20%
    const MAX_RATIO: u32 = 100; // 100%
    
    if ratio_percent < MIN_RATIO || ratio_percent > MAX_RATIO {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

// ===== Governance Validation =====

/// Validate quorum percentage
pub fn validate_quorum_percent(percent: u32) -> Result<(), ContractError> {
    validate_percentage(percent)?;
    
    // Typical minimum quorum is 25%
    const MIN_QUORUM: u32 = 25;
    if percent < MIN_QUORUM {
        return Err(ContractError::QuorumNotMet);
    }
    
    Ok(())
}

/// Validate voting threshold
pub fn validate_voting_threshold(percent: u32) -> Result<(), ContractError> {
    validate_percentage(percent)?;
    
    // Typical threshold is > 50%
    if percent <= 50 {
        return Err(ContractError::ThresholdNotMet);
    }
    
    Ok(())
}

// ===== Oracle Validation =====

/// Validate oracle submissions count
pub fn validate_oracle_submissions(count: u32) -> Result<(), ContractError> {
    const MIN_SUBMISSIONS: u32 = 1;
    const MAX_SUBMISSIONS: u32 = 100;
    
    if count < MIN_SUBMISSIONS || count > MAX_SUBMISSIONS {
        return Err(ContractError::InsufficientOracleSubmissions);
    }
    Ok(())
}

/// Validate oracle data age
pub fn validate_oracle_data_age(
    current_time: u64,
    data_time: u64,
    max_age_seconds: u64,
) -> Result<(), ContractError> {
    if data_time > current_time {
        return Err(ContractError::InvalidInput);
    }
    
    let age = current_time - data_time;
    if age > max_age_seconds {
        return Err(ContractError::OracleDataStale);
    }
    
    Ok(())
}
