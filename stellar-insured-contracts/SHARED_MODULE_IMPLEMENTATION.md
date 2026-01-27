# Shared Module Implementation Guide

## Overview

This document provides implementation details for the shared types, errors, and constants module that enables consistency and safety across all Stellar Insured contracts.

## Architecture

### Module Organization

```
shared/
├── src/
│   ├── lib.rs          # Main library with re-exports
│   ├── errors.rs       # ContractError enum (145+ codes)
│   ├── types.rs        # Status enums and data structures
│   ├── constants.rs    # Configuration constants and helpers
│   └── validation.rs   # Validation and arithmetic functions
└── Cargo.toml
```

### Design Principles

1. **No-std Compatibility** - Works in Soroban's constrained environment
2. **Type Safety** - Rust's type system prevents invalid states
3. **Error Organization** - Grouped by category with unique codes
4. **Constants Over Magic Numbers** - All limits defined in one place
5. **Validation Functions** - Reusable, testable validation logic

## Error Design

### Error Code Organization

Errors are organized in ranges by category:

| Range | Category | Count |
|-------|----------|-------|
| 1-19 | General/Authorization | 15 |
| 20-39 | Policy | 7 |
| 40-59 | Claim | 10 |
| 60-79 | Oracle | 6 |
| 80-99 | Governance | 10 |
| 100-119 | Treasury | 5 |
| 120-139 | Slashing | 4 |
| 140-159 | Risk Pool | 5 |

### Adding New Errors

When adding a new error:

1. Choose appropriate range
2. Assign unique code number
3. Add variant to enum
4. Add message to `.message()` method
5. Update any From implementations
6. Document in SHARED_TYPES_ERRORS_CONSTANTS.md

```rust
// In errors.rs
pub enum ContractError {
    // Existing...
    MyNewError = 150,  // Pick appropriate range
}

// In message() method
ContractError::MyNewError => "My error description",
```

## Types Design

### Status Enums

Status enums represent lifecycle states with constrained transitions:

**PolicyStatus**:
```
Active ──┬──→ Expired (terminal)
         └──→ Cancelled (terminal)
         └──→ Claimed
```

**ClaimStatus**:
```
Submitted ──→ UnderReview ──┬──→ Approved ──→ Settled (terminal)
                            └──→ Rejected (terminal)
```

**ProposalStatus**:
```
Active ──┬──→ Passed ──→ Executed (terminal)
         ├──→ Rejected (terminal)
         └──→ Expired (terminal)
```

To validate transitions:

```rust
impl PolicyStatus {
    pub fn can_transition_to(self, next: PolicyStatus) -> bool {
        match (self, next) {
            (PolicyStatus::Active, PolicyStatus::Expired) => true,
            (PolicyStatus::Active, PolicyStatus::Cancelled) => true,
            (PolicyStatus::Expired, _) => false,
            (PolicyStatus::Cancelled, _) => false,
            _ => false,
        }
    }
}
```

### Data Structures

Common data structures include:

1. **ClaimEvidence** - Immutable proof (hash-based)
2. **VoteRecord** - Governance voting record
3. **OracleConfig** - Oracle validation settings
4. **RiskMetrics** - Pool health metrics
5. **PolicyMetadata** - Policy tracking data
6. **ClaimMetadata** - Claim tracking data
7. **TreasuryAllocation** - Fund allocation record

## Constants

### Amount Constraints

```rust
// Coverage bounds (1 unit to 1M units)
const MIN_COVERAGE_AMOUNT: i128 = 1_000_000;
const MAX_COVERAGE_AMOUNT: i128 = 1_000_000_000_000_000;

// Premium bounds (0.1 units to 100k units)
const MIN_PREMIUM_AMOUNT: i128 = 100_000;
const MAX_PREMIUM_AMOUNT: i128 = 100_000_000_000_000;

// Duration bounds (1 to 365 days)
const MIN_POLICY_DURATION_DAYS: u32 = 1;
const MAX_POLICY_DURATION_DAYS: u32 = 365;
```

### Time-based Constants

```rust
// Using ONE_DAY_SECONDS as base unit
const ONE_DAY_SECONDS: u64 = 86_400;
const CLAIM_GRACE_PERIOD_SECONDS: u64 = 30 * ONE_DAY_SECONDS;
const DEFAULT_VOTING_PERIOD_SECONDS: u64 = 7 * ONE_DAY_SECONDS;
```

### Governance Parameters

```rust
// Quorum and thresholds
const DEFAULT_MIN_QUORUM_PERCENT: u32 = 25;      // 25%
const DEFAULT_APPROVAL_THRESHOLD_PERCENT: u32 = 50; // >50%
const DEFAULT_EXECUTIVE_QUORUM_PERCENT: u32 = 66;   // 66%

// Basis points (for precision)
const MAX_BASIS_POINTS: u32 = 10_000;             // 100%
const BASIS_POINTS_PER_PERCENT: u32 = 100;
```

### Risk Management

```rust
// Reserve ratio
const MIN_RESERVE_RATIO_PERCENT: u32 = 20;       // 20%
const TARGET_RESERVE_RATIO_PERCENT: u32 = 50;    // 50%

// Loss tolerance
const MAX_ALLOWED_LOSS_RATIO_PERCENT: u32 = 80;  // 80%
const CRITICAL_RESERVE_THRESHOLD_PERCENT: u32 = 10; // 10%
```

## Validation Functions

### Architecture

Validation functions follow a consistent pattern:

```rust
pub fn validate_something(value: T) -> Result<(), ContractError> {
    if !is_valid(value) {
        return Err(ContractError::SpecificError);
    }
    Ok(())
}
```

### Categories

1. **Address Validation**
   - `validate_address()` - Check address validity
   - `validate_addresses_different()` - Ensure addresses differ

2. **Amount Validation**
   - `validate_positive_amount()` - Amount > 0
   - `validate_amount_in_bounds()` - Amount in [min, max]
   - `validate_coverage_amount()` - Specific bounds
   - `validate_premium_amount()` - Specific bounds
   - `validate_sufficient_funds()` - Balance check

3. **Time Validation**
   - `validate_future_timestamp()` - Timestamp in future
   - `validate_past_timestamp()` - Timestamp in past
   - `validate_time_range()` - start < end
   - `validate_duration_days()` - Duration in bounds

4. **State Validation**
   - `validate_not_paused()` - Contract not paused
   - `validate_initialized()` - Contract initialized
   - `validate_not_initialized()` - Contract not yet initialized

5. **Arithmetic Validation**
   - `safe_add()` - Addition with overflow check
   - `safe_sub()` - Subtraction with underflow check
   - `safe_mul()` - Multiplication with overflow check
   - `safe_div()` - Division with zero check

### Batch Validation

```rust
pub fn validate_all(conditions: &[(bool, ContractError)]) -> Result<(), ContractError> {
    for &(is_valid, error) in conditions {
        if !is_valid {
            return Err(error);
        }
    }
    Ok(())
}

// Usage
validate_all(&[
    (amount > 0, ContractError::InvalidInput),
    (amount <= MAX, ContractError::InvalidInput),
    (sender != recipient, ContractError::InvalidInput),
])?;
```

## Integration with Contracts

### Using Shared Types

```rust
use shared::{ContractError, PolicyStatus, validate_address};

#[contractimpl]
pub fn create_policy(
    env: Env,
    holder: Address,
    coverage: i128,
) -> Result<u64, ContractError> {
    // Validate inputs
    validate_address(&env, &holder)?;
    validate_coverage_amount(coverage)?;
    
    // Use shared types
    let policy = Policy {
        holder,
        coverage_amount: coverage,
        status: PolicyStatus::Active,
        // ...
    };
    
    Ok(policy_id)
}
```

### Implementing Transitions

```rust
// Validate state transition
if !current_status.can_transition_to(new_status) {
    return Err(ContractError::InvalidStateTransition);
}

// Update status
policy.status = new_status;
```

## Best Practices

### 1. Always Validate Input

```rust
// ✅ Good
pub fn submit_claim(env: Env, claim_id: u64, amount: i128) -> Result<(), ContractError> {
    validate_positive_amount(amount)?;
    // ... rest of logic
}

// ❌ Bad
pub fn submit_claim(env: Env, claim_id: u64, amount: i128) -> Result<(), ContractError> {
    // No validation - could cause issues later
}
```

### 2. Use Safe Arithmetic

```rust
// ✅ Good
let total = safe_add(amount1, amount2)?;

// ❌ Bad
let total = amount1 + amount2; // Could overflow!
```

### 3. Use Shared Constants

```rust
// ✅ Good
if coverage_amount < MIN_COVERAGE_AMOUNT {
    return Err(ContractError::InvalidCoverageAmount);
}

// ❌ Bad
if coverage_amount < 1_000_000 {
    return Err(ContractError::InvalidCoverageAmount);
}
```

### 4. Document Error Handling

```rust
/// Create a policy
///
/// # Errors
/// - `InvalidCoverageAmount` - Coverage out of bounds
/// - `InvalidPremiumAmount` - Premium out of bounds
/// - `InvalidDuration` - Duration out of bounds
pub fn create_policy(...) -> Result<u64, ContractError> {
    // ...
}
```

## Testing

### Error Testing

```rust
#[test]
fn test_invalid_coverage() {
    let result = validate_coverage_amount(500); // Below minimum
    assert_eq!(result, Err(ContractError::InvalidCoverageAmount));
}
```

### Validation Testing

```rust
#[test]
fn test_safe_arithmetic() {
    let result = safe_add(i128::MAX, 1);
    assert_eq!(result, Err(ContractError::Overflow));
}
```

## Performance Considerations

1. **Constants** - Zero runtime cost (compile-time constants)
2. **Validation** - Minimal overhead, prevents costly errors later
3. **No Allocations** - Uses no-std, minimal memory usage
4. **Inlining** - Small functions automatically inlined

## Future Extensions

### Planned Additions

1. **Percentage Calculations** - Helper functions for discount/fee calculations
2. **Time Calculations** - Period end dates, durations
3. **Recovery Helpers** - Graceful degradation patterns
4. **Batch Operations** - Multi-entity validation
5. **Metrics** - Performance measurement helpers

### Adding Features

When adding new features:

1. Add to appropriate module
2. Follow existing patterns
3. Add comprehensive documentation
4. Include usage examples
5. Test thoroughly
6. Update relevant markdown files

## Module Maintenance

### Regular Tasks

1. **Review Error Codes** - Ensure no conflicts
2. **Verify Constants** - Check against spec
3. **Update Documentation** - Keep docs in sync
4. **Test Coverage** - Maintain comprehensive tests
5. **Performance Audit** - Monitor contract size

### Changelog

Track changes in [SHARED_MODULE_CHANGELOG.md]

## Troubleshooting

### "Error code conflict"

Error codes must be unique. Check existing codes before adding new ones.

### "Arithmetic overflow/underflow"

Always use `safe_*` functions for untrusted input:
```rust
let result = safe_mul(amount, percent)?; // Protected
```

### "Validation chain failure"

Break validation into smaller steps and check each one:
```rust
validate_positive_amount(amount)?;
validate_coverage_amount(amount)?;
validate_time_range(start, end)?;
```

## References

- [Shared Types, Errors & Constants Module](./SHARED_TYPES_ERRORS_CONSTANTS.md)
- [Main Contract Documentation](./README.md)
- [Soroban SDK Documentation](https://docs.rs/soroban-sdk/)
