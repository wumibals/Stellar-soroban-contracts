# Shared Types, Errors & Constants Module

## Overview

The **shared** module provides a unified library of reusable types, errors, constants, and validation helpers used across all Stellar Insured contracts. This ensures consistency, safety, and maintainability across the entire insurance protocol.

## Module Structure

```
shared/
├── src/
│   ├── lib.rs           # Main library entry point
│   ├── errors.rs        # Common error types
│   ├── types.rs         # Shared data types and enums
│   ├── constants.rs     # Configuration constants
│   └── validation.rs    # Validation helper utilities
├── Cargo.toml           # Module dependencies
└── README.md            # Documentation
```

## Key Components

### 1. Error Types (`errors.rs`)

A comprehensive error enum with 145+ error codes organized by category:

**General/Authorization (1-19)**
- `Unauthorized` - Caller not authorized
- `Paused` - Contract is paused
- `InvalidInput` - Invalid input provided
- `InsufficientFunds` - Not enough funds
- `NotFound` - Resource not found
- `AlreadyExists` - Resource already exists
- `InvalidState` - Invalid state for operation
- `Overflow` / `Underflow` - Arithmetic errors
- `InvalidRole` / `RoleNotFound` - Role-based errors
- `NotTrustedContract` - Contract not trusted

**Policy-Specific (20-39)**
- `PolicyNotFound` - Policy doesn't exist
- `InvalidPolicyState` - Invalid state for operation
- `InvalidCoverageAmount` - Coverage out of bounds
- `InvalidPremiumAmount` - Premium out of bounds
- `InvalidDuration` - Duration out of bounds
- `CannotRenewPolicy` - Cannot renew policy
- `InvalidStateTransition` - Invalid state change

**Claim-Specific (40-59)**
- `ClaimNotFound` - Claim doesn't exist
- `InvalidClaimState` - Invalid state for operation
- `ClaimAmountExceedsCoverage` - Claim exceeds coverage
- `ClaimPeriodExpired` - Can't submit claim after period
- `PolicyCoverageExpired` - Policy coverage expired
- `EvidenceError` - Evidence-related errors

**Oracle-Specific (60-79)**
- `OracleValidationFailed` - Oracle validation failed
- `InsufficientOracleSubmissions` - Not enough submissions
- `OracleDataStale` - Data is too old
- `OracleOutlierDetected` - Data is outlier
- `OracleNotConfigured` - Oracle not set up

**Governance (80-99)**
- `VotingPeriodEnded` - Voting period over
- `AlreadyVoted` - Already voted on proposal
- `ProposalNotActive` - Proposal not active
- `QuorumNotMet` - Not enough voters
- `ThresholdNotMet` - Didn't reach threshold

**Treasury (100-119)**
- `TreasuryFundNotFound` - Fund not found
- `InsufficientTreasuryBalance` - Not enough balance
- `InvalidAllocation` - Invalid allocation
- `TreasuryLocked` - Treasury is locked

**Slashing (120-139)**
- `ValidatorNotFound` - Validator not found
- `InvalidSlashingAmount` - Amount out of bounds
- `SlashingAlreadyExecuted` - Already executed
- `SlashingPeriodNotActive` - Not in slashing period

**Risk Pool (140-159)**
- `RiskPoolNotFound` - Risk pool not found
- `InvalidRiskPoolState` - Invalid state
- `InsufficientRiskPoolBalance` - Not enough balance
- `RiskPoolLocked` - Pool is locked

Each error includes a `.message()` method for human-readable descriptions.

### 2. Shared Types (`types.rs`)

**Status Enums** - Represent lifecycle states with valid transitions:

```rust
// Policy status with allowed transitions: Active → Expired/Cancelled
pub enum PolicyStatus {
    Active,
    Expired,
    Cancelled,
    Claimed,
}

// Claim status with allowed transitions: Submitted → UnderReview → Approved/Rejected → Settled
pub enum ClaimStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Settled,
}

// Governance proposal status
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Expired,
}

// Proposal types
pub enum ProposalType {
    ParameterChange,
    ContractUpgrade,
    SlashingAction,
    TreasuryAllocation,
    EmergencyAction,
}

// Vote type
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

// Risk pool status
pub enum RiskPoolStatus {
    Active,
    Paused,
    Emergency,
    Closed,
}
```

**Data Structures**:

```rust
// Claim evidence (hash-only for immutability)
pub struct ClaimEvidence {
    pub claim_id: BytesN<32>,
    pub evidence_hash: BytesN<32>,
    pub submitter: Address,
    pub submitted_at: u64,
}

// Vote record
pub struct VoteRecord {
    pub proposal_id: u64,
    pub voter: Address,
    pub vote: VoteType,
    pub voting_power: i128,
    pub voted_at: u64,
}

// Oracle configuration
pub struct OracleConfig {
    pub oracle_contract: Address,
    pub require_oracle_validation: bool,
    pub min_oracle_submissions: u32,
    pub max_data_age: u64,
    pub max_deviation_bps: u32,
}

// Risk metrics
pub struct RiskMetrics {
    pub total_value_at_risk: i128,
    pub reserve_balance: i128,
    pub reserve_ratio_percent: u32,
    pub total_claims_paid: i128,
    pub loss_ratio_percent: u32,
}

// Policy and claim metadata
pub struct PolicyMetadata { ... }
pub struct ClaimMetadata { ... }
pub struct TreasuryAllocation { ... }
```

### 3. Constants (`constants.rs`)

**Amount Constraints**:
```rust
const MIN_COVERAGE_AMOUNT: i128 = 1_000_000;           // 1 unit
const MAX_COVERAGE_AMOUNT: i128 = 1_000_000_000_000_000;
const MIN_PREMIUM_AMOUNT: i128 = 100_000;              // 0.1 units
const MAX_PREMIUM_AMOUNT: i128 = 100_000_000_000_000;
const MIN_POLICY_DURATION_DAYS: u32 = 1;
const MAX_POLICY_DURATION_DAYS: u32 = 365;
```

**Time Constants**:
```rust
const ONE_DAY_SECONDS: u64 = 86_400;
const ONE_MONTH_SECONDS: u64 = 2_629_746;
const ONE_YEAR_SECONDS: u64 = 31_557_600;
const CLAIM_GRACE_PERIOD_SECONDS: u64 = 30 * ONE_DAY_SECONDS;
const DEFAULT_VOTING_PERIOD_SECONDS: u64 = 7 * ONE_DAY_SECONDS;
```

**Governance Parameters**:
```rust
const DEFAULT_MIN_QUORUM_PERCENT: u32 = 25;
const DEFAULT_APPROVAL_THRESHOLD_PERCENT: u32 = 50;
const MAX_PERCENTAGE: u32 = 100;
const MAX_BASIS_POINTS: u32 = 10_000;
```

**Risk Management**:
```rust
const MIN_RESERVE_RATIO_PERCENT: u32 = 20;
const MAX_RESERVE_RATIO_PERCENT: u32 = 100;
const TARGET_RESERVE_RATIO_PERCENT: u32 = 50;
const MAX_ALLOWED_LOSS_RATIO_PERCENT: u32 = 80;
const CRITICAL_RESERVE_THRESHOLD_PERCENT: u32 = 10;
```

**Oracle Settings**:
```rust
const DEFAULT_MAX_ORACLE_DATA_AGE: u64 = 24 * ONE_DAY_SECONDS;
const DEFAULT_MIN_ORACLE_SUBMISSIONS: u32 = 3;
const DEFAULT_MAX_ORACLE_DEVIATION_BPS: u32 = 500; // 5%
```

**Helper Functions**:
```rust
// Validation helpers
pub fn is_coverage_amount_valid(amount: i128) -> bool
pub fn is_premium_amount_valid(amount: i128) -> bool
pub fn is_duration_valid(duration_days: u32) -> bool
pub fn is_percentage_valid(percent: u32) -> bool

// Conversion helpers
pub fn percent_to_basis_points(percent: u32) -> u32
pub fn basis_points_to_percent(bps: u32) -> u32

// Calculation helpers
pub fn calculate_percentage(amount: i128, percent: u32) -> i128
pub fn calculate_basis_points(amount: i128, bps: u32) -> i128

// Safe arithmetic
pub fn safe_add(a: i128, b: i128) -> Option<i128>
pub fn safe_sub(a: i128, b: i128) -> Option<i128>
pub fn safe_mul(a: i128, b: i128) -> Option<i128>
pub fn safe_div(a: i128, b: i128) -> Option<i128>
```

### 4. Validation Helpers (`validation.rs`)

Comprehensive validation functions organized by category:

**Address Validation**:
```rust
pub fn validate_address(env: &Env, address: &Address) -> Result<(), ContractError>
pub fn validate_addresses(env: &Env, addresses: &[Address]) -> Result<(), ContractError>
pub fn validate_addresses_different(addr1: &Address, addr2: &Address) -> Result<(), ContractError>
```

**Amount Validation**:
```rust
pub fn validate_positive_amount(amount: i128) -> Result<(), ContractError>
pub fn validate_non_negative_amount(amount: i128) -> Result<(), ContractError>
pub fn validate_amount_in_bounds(amount: i128, min: i128, max: i128) -> Result<(), ContractError>
pub fn validate_coverage_amount(amount: i128) -> Result<(), ContractError>
pub fn validate_premium_amount(amount: i128) -> Result<(), ContractError>
pub fn validate_sufficient_funds(balance: i128, required: i128) -> Result<(), ContractError>
```

**Time Validation**:
```rust
pub fn validate_future_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError>
pub fn validate_past_timestamp(current_time: u64, timestamp: u64) -> Result<(), ContractError>
pub fn validate_time_range(start_time: u64, end_time: u64) -> Result<(), ContractError>
pub fn validate_duration_days(duration_days: u32) -> Result<(), ContractError>
```

**Percentage Validation**:
```rust
pub fn validate_percentage(percent: u32) -> Result<(), ContractError>
pub fn validate_basis_points(bps: u32) -> Result<(), ContractError>
pub fn validate_oracle_deviation(deviation_bps: u32) -> Result<(), ContractError>
```

**State Validation**:
```rust
pub fn validate_not_paused(is_paused: bool) -> Result<(), ContractError>
pub fn validate_initialized(is_initialized: bool) -> Result<(), ContractError>
pub fn validate_not_initialized(is_initialized: bool) -> Result<(), ContractError>
```

**Arithmetic Validation**:
```rust
pub fn safe_add(a: i128, b: i128) -> Result<i128, ContractError>
pub fn safe_sub(a: i128, b: i128) -> Result<i128, ContractError>
pub fn safe_mul(a: i128, b: i128) -> Result<i128, ContractError>
pub fn safe_div(a: i128, b: i128) -> Result<i128, ContractError>
```

**Governance Validation**:
```rust
pub fn validate_quorum_percent(percent: u32) -> Result<(), ContractError>
pub fn validate_voting_threshold(percent: u32) -> Result<(), ContractError>
```

**Oracle Validation**:
```rust
pub fn validate_oracle_submissions(count: u32) -> Result<(), ContractError>
pub fn validate_oracle_data_age(current_time: u64, data_time: u64, max_age_seconds: u64) -> Result<(), ContractError>
```

## Usage Examples

### Basic Import and Usage

```rust
// In your contract
use shared::{
    ContractError,
    PolicyStatus, ClaimStatus, ProposalStatus,
    validate_address, validate_coverage_amount,
    calculate_percentage,
    MIN_COVERAGE_AMOUNT, MAX_COVERAGE_AMOUNT,
};

// Validate input
validate_address(&env, &policy_holder)?;
validate_coverage_amount(coverage_amount)?;

// Use shared types
let status = PolicyStatus::Active;
match status {
    PolicyStatus::Active => { /* ... */ },
    PolicyStatus::Expired => { /* ... */ },
    _ => {},
}

// Use constants
if coverage_amount < MIN_COVERAGE_AMOUNT {
    return Err(ContractError::InvalidCoverageAmount);
}

// Safe arithmetic
let fee = calculate_percentage(amount, 5)?; // 5% fee
```

### Error Handling

```rust
// Custom error messages
if insufficient {
    return Err(ContractError::InsufficientFunds);
}

// Get error message
let msg = ContractError::InsufficientFunds.message(); // "Insufficient funds"
```

### Validation Chain

```rust
use shared::validation::validate_all;

validate_all(&[
    (amount > 0, ContractError::InvalidInput),
    (amount <= MAX_AMOUNT, ContractError::InvalidInput),
    (sender != recipient, ContractError::InvalidInput),
])?;
```

## Benefits

✅ **Consistency** - All contracts use the same error codes, types, and constants
✅ **Safety** - Compiled-checked state transitions and validated calculations
✅ **Maintainability** - Centralized definitions prevent duplication
✅ **Reusability** - Common validation functions available across all contracts
✅ **Documentation** - Well-documented types and functions with examples
✅ **Performance** - No-std, zero-cost abstractions for Soroban

## Integration Checklist

- [x] Define error types with organized error codes
- [x] Define status enums with valid transitions
- [x] Define data structures for common concepts
- [x] Define constants for validation limits
- [x] Provide validation helper functions
- [x] Provide arithmetic helpers with overflow protection
- [x] Document all types and functions
- [x] Export commonly used items from lib.rs

## Related Documents

- [SHARED_MODULE_IMPLEMENTATION.md](./SHARED_MODULE_IMPLEMENTATION.md) - Detailed implementation notes
- [Validation Examples](./VALIDATION_EXAMPLES.md) - Common validation patterns
- [Error Code Reference](./ERROR_CODES.md) - Complete error code listing
