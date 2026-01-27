//! Common constants used across insurance contracts
//!
//! This module defines all shared constants for validation, limits, and configuration
//! to ensure consistency across all contracts.

// ===== Numeric Constants =====

/// Minimum coverage amount (1 unit with 6 decimals)
pub const MIN_COVERAGE_AMOUNT: i128 = 1_000_000;

/// Maximum coverage amount (1M units)
pub const MAX_COVERAGE_AMOUNT: i128 = 1_000_000_000_000_000;

/// Minimum premium amount (0.1 units)
pub const MIN_PREMIUM_AMOUNT: i128 = 100_000;

/// Maximum premium amount (100k units)
pub const MAX_PREMIUM_AMOUNT: i128 = 100_000_000_000_000;

/// Minimum policy duration in days
pub const MIN_POLICY_DURATION_DAYS: u32 = 1;

/// Maximum policy duration in days
pub const MAX_POLICY_DURATION_DAYS: u32 = 365;

// ===== Time Constants (in seconds) =====

/// One day in seconds (86400)
pub const ONE_DAY_SECONDS: u64 = 86_400;

/// One month in seconds (approximately 30.44 days)
pub const ONE_MONTH_SECONDS: u64 = 2_629_746;

/// One year in seconds (approximately 365.25 days)
pub const ONE_YEAR_SECONDS: u64 = 31_557_600;

/// Grace period for claim submission after policy expiry (30 days)
pub const CLAIM_GRACE_PERIOD_SECONDS: u64 = 30 * ONE_DAY_SECONDS;

/// Default voting period duration (7 days)
pub const DEFAULT_VOTING_PERIOD_SECONDS: u64 = 7 * ONE_DAY_SECONDS;

/// Default proposal expiry (30 days)
pub const PROPOSAL_EXPIRY_SECONDS: u64 = 30 * ONE_DAY_SECONDS;

// ===== Governance Constants =====

/// Default minimum quorum percentage (25%)
pub const DEFAULT_MIN_QUORUM_PERCENT: u32 = 25;

/// Default approval threshold percentage (50%)
pub const DEFAULT_APPROVAL_THRESHOLD_PERCENT: u32 = 50;

/// Default executive quorum percentage (66%)
pub const DEFAULT_EXECUTIVE_QUORUM_PERCENT: u32 = 66;

/// Maximum percentage for any vote (100%)
pub const MAX_PERCENTAGE: u32 = 100;

/// Basis points per percentage (100)
pub const BASIS_POINTS_PER_PERCENT: u32 = 100;

/// Maximum basis points (10000 = 100%)
pub const MAX_BASIS_POINTS: u32 = 10_000;

// ===== Oracle Constants =====

/// Default maximum oracle data age (24 hours)
pub const DEFAULT_MAX_ORACLE_DATA_AGE: u64 = 24 * ONE_DAY_SECONDS;

/// Default minimum oracle submissions
pub const DEFAULT_MIN_ORACLE_SUBMISSIONS: u32 = 3;

/// Default maximum deviation in basis points (500 = 5%)
pub const DEFAULT_MAX_ORACLE_DEVIATION_BPS: u32 = 500;

/// Minimum oracle submissions
pub const MIN_ORACLE_SUBMISSIONS: u32 = 1;

/// Maximum oracle submissions to consider
pub const MAX_ORACLE_SUBMISSIONS: u32 = 100;

// ===== Risk Management Constants =====

/// Minimum reserve ratio percentage (20%)
pub const MIN_RESERVE_RATIO_PERCENT: u32 = 20;

/// Maximum reserve ratio percentage (100%)
pub const MAX_RESERVE_RATIO_PERCENT: u32 = 100;

/// Target reserve ratio percentage (50%)
pub const TARGET_RESERVE_RATIO_PERCENT: u32 = 50;

/// Maximum loss ratio percentage allowed (80%)
pub const MAX_ALLOWED_LOSS_RATIO_PERCENT: u32 = 80;

/// Critical reserve threshold (10%)
pub const CRITICAL_RESERVE_THRESHOLD_PERCENT: u32 = 10;

// ===== Slashing Constants =====

/// Minimum slashing amount (0.1 units)
pub const MIN_SLASHING_AMOUNT: i128 = 100_000;

/// Maximum slashing percentage (10%)
pub const MAX_SLASHING_PERCENT: u32 = 10;

/// Slashing cooldown period (7 days)
pub const SLASHING_COOLDOWN_SECONDS: u64 = 7 * ONE_DAY_SECONDS;

// ===== Treasury Constants =====

/// Minimum allocation amount (0.01 units)
pub const MIN_ALLOCATION_AMOUNT: i128 = 10_000;

/// Maximum allocation percentage per transaction (10%)
pub const MAX_ALLOCATION_PERCENT_PER_TX: u32 = 10;

/// Treasury lock-up period (7 days)
pub const TREASURY_LOCKUP_SECONDS: u64 = 7 * ONE_DAY_SECONDS;

// ===== Decimals Constants =====

/// Standard Stellar asset decimals
pub const STELLAR_DECIMAL_PLACES: u32 = 7;

/// Stroops per unit (10^7)
pub const STROOPS_PER_UNIT: i128 = 10_000_000;

// ===== Pagination Constants =====

/// Maximum results per page for paginated queries
pub const MAX_PAGE_SIZE: u32 = 1000;

/// Default page size
pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Minimum page size
pub const MIN_PAGE_SIZE: u32 = 1;

// ===== String Length Constants =====

/// Maximum length for policy metadata
pub const MAX_METADATA_LENGTH: u32 = 1024;

/// Maximum length for claim description
pub const MAX_DESCRIPTION_LENGTH: u32 = 2048;

/// Maximum length for evidence metadata
pub const MAX_EVIDENCE_METADATA_LENGTH: u32 = 512;

// ===== Retry and Timeout Constants =====

/// Maximum retry attempts
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Timeout for external contract calls (in seconds)
pub const EXTERNAL_CALL_TIMEOUT_SECONDS: u64 = 60;

// ===== Validation Helper Functions =====

/// Validate that an amount is within acceptable bounds
///
/// # Arguments
/// * `amount` - The amount to validate
/// * `min` - Minimum allowed amount
/// * `max` - Maximum allowed amount
///
/// # Returns
/// `true` if amount is within bounds, `false` otherwise
pub fn is_amount_valid(amount: i128, min: i128, max: i128) -> bool {
    amount >= min && amount <= max
}

/// Validate coverage amount
pub fn is_coverage_amount_valid(amount: i128) -> bool {
    is_amount_valid(amount, MIN_COVERAGE_AMOUNT, MAX_COVERAGE_AMOUNT)
}

/// Validate premium amount
pub fn is_premium_amount_valid(amount: i128) -> bool {
    is_amount_valid(amount, MIN_PREMIUM_AMOUNT, MAX_PREMIUM_AMOUNT)
}

/// Validate policy duration
pub fn is_duration_valid(duration_days: u32) -> bool {
    duration_days >= MIN_POLICY_DURATION_DAYS && duration_days <= MAX_POLICY_DURATION_DAYS
}

/// Validate percentage (0-100)
pub fn is_percentage_valid(percent: u32) -> bool {
    percent <= 100
}

/// Validate basis points (0-10000)
pub fn is_basis_points_valid(bps: u32) -> bool {
    bps <= MAX_BASIS_POINTS
}

/// Convert percentage to basis points
pub fn percent_to_basis_points(percent: u32) -> u32 {
    percent * BASIS_POINTS_PER_PERCENT
}

/// Convert basis points to percentage
pub fn basis_points_to_percent(bps: u32) -> u32 {
    bps / BASIS_POINTS_PER_PERCENT
}

/// Calculate percentage of an amount
pub fn calculate_percentage(amount: i128, percent: u32) -> i128 {
    if percent == 0 {
        return 0;
    }
    (amount * (percent as i128)) / 100
}

/// Calculate basis points of an amount
pub fn calculate_basis_points(amount: i128, bps: u32) -> i128 {
    if bps == 0 {
        return 0;
    }
    (amount * (bps as i128)) / MAX_BASIS_POINTS as i128
}

/// Safely add two amounts with overflow protection
pub fn safe_add(a: i128, b: i128) -> Option<i128> {
    a.checked_add(b)
}

/// Safely subtract two amounts with underflow protection
pub fn safe_sub(a: i128, b: i128) -> Option<i128> {
    a.checked_sub(b)
}

/// Safely multiply two amounts with overflow protection
pub fn safe_mul(a: i128, b: i128) -> Option<i128> {
    a.checked_mul(b)
}

/// Safely divide two amounts
pub fn safe_div(a: i128, b: i128) -> Option<i128> {
    if b == 0 {
        return None;
    }
    a.checked_div(b)
}
