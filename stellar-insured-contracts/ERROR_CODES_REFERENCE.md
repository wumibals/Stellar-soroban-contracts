# Error Code Reference

Complete listing of all error codes used in the Stellar Insured contracts system.

## Error Code Ranges

- **1-19**: General/Authorization errors
- **20-39**: Policy-specific errors
- **40-59**: Claim-specific errors
- **60-79**: Oracle-specific errors
- **80-99**: Governance errors
- **100-119**: Treasury errors
- **120-139**: Slashing errors
- **140-159**: Risk Pool errors

## Detailed Error List

### General/Authorization Errors (1-19)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 1 | `Unauthorized` | Caller is not authorized | Check permissions, use authorized account |
| 2 | `Paused` | Contract is paused | Wait for contract to be unpaused |
| 3 | `InvalidInput` | Invalid input provided | Check input parameters |
| 4 | `InsufficientFunds` | Insufficient funds for operation | Deposit more funds |
| 5 | `NotFound` | Requested resource not found | Use existing resource ID |
| 6 | `AlreadyExists` | Resource already exists | Use unique identifier |
| 7 | `InvalidState` | Invalid state for operation | Check contract state |
| 8 | `Overflow` | Arithmetic overflow occurred | Use smaller numbers |
| 9 | `NotInitialized` | Contract not initialized | Call initialize function |
| 10 | `AlreadyInitialized` | Contract already initialized | Only initialize once |
| 11 | `InvalidRole` | Invalid role or permission | Use valid role |
| 12 | `RoleNotFound` | Role not found | Assign role first |
| 13 | `NotTrustedContract` | Contract not trusted | Register contract as trusted |
| 14 | `InvalidAddress` | Invalid address format | Use valid address |
| 15 | `Underflow` | Arithmetic underflow occurred | Use larger numbers |

### Policy-Specific Errors (20-39)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 20 | `PolicyNotFound` | Policy doesn't exist | Use existing policy ID |
| 21 | `InvalidPolicyState` | Invalid state for operation | Check policy status |
| 22 | `InvalidCoverageAmount` | Coverage out of bounds | Use amount between 1-1M units |
| 23 | `InvalidPremiumAmount` | Premium out of bounds | Use amount between 0.1-100k units |
| 24 | `InvalidDuration` | Duration out of bounds | Use 1-365 days |
| 25 | `CannotRenewPolicy` | Cannot renew policy | Policy must be active |
| 26 | `InvalidStateTransition` | Invalid state transition | Check valid transitions |

### Claim-Specific Errors (40-59)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 40 | `ClaimNotFound` | Claim doesn't exist | Use existing claim ID |
| 41 | `InvalidClaimState` | Invalid state for operation | Check claim status |
| 42 | `ClaimAmountExceedsCoverage` | Claim exceeds coverage | Reduce claim amount |
| 43 | `ClaimPeriodExpired` | Can't submit claim after period | Submit within grace period |
| 44 | `CannotSubmitClaim` | Cannot submit claim for policy | Check policy eligibility |
| 45 | `PolicyCoverageExpired` | Policy coverage has expired | Renew or create new policy |
| 46 | `EvidenceError` | Evidence-related error | Check evidence format |
| 47 | `EvidenceAlreadyExists` | Evidence already exists | Use unique evidence ID |
| 48 | `EvidenceNotFound` | Evidence not found | Submit evidence first |
| 49 | `InvalidEvidenceHash` | Invalid evidence hash | Use SHA-256 hash |

### Oracle-Specific Errors (60-79)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 60 | `OracleValidationFailed` | Oracle validation failed | Submit valid oracle data |
| 61 | `InsufficientOracleSubmissions` | Not enough oracle submissions | Wait for more submissions |
| 62 | `OracleDataStale` | Oracle data is too old | Update with fresh data |
| 63 | `OracleOutlierDetected` | Oracle data is outlier | Submit data within deviation bounds |
| 64 | `OracleNotConfigured` | Oracle not configured | Configure oracle contract |
| 65 | `InvalidOracleContract` | Invalid oracle contract | Use valid oracle address |

### Governance Errors (80-99)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 80 | `VotingPeriodEnded` | Voting period has ended | Voting is closed |
| 81 | `AlreadyVoted` | Already voted on proposal | Can't vote twice |
| 82 | `ProposalNotActive` | Proposal is not active | Use active proposal |
| 83 | `QuorumNotMet` | Quorum not met | Need more votes |
| 84 | `ThresholdNotMet` | Threshold not met | Need more votes in favor |
| 85 | `ProposalNotFound` | Proposal not found | Use existing proposal ID |
| 86 | `InvalidProposalType` | Invalid proposal type | Use valid proposal type |
| 87 | `SlashingContractNotSet` | Slashing contract not set | Configure slashing contract |
| 88 | `SlashingExecutionFailed` | Slashing execution failed | Check slashing parameters |

### Treasury Errors (100-119)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 100 | `TreasuryFundNotFound` | Treasury fund not found | Create fund first |
| 101 | `InsufficientTreasuryBalance` | Insufficient treasury balance | Deposit more funds |
| 102 | `InvalidAllocation` | Invalid allocation | Check allocation parameters |
| 103 | `InvalidDistribution` | Invalid distribution | Check distribution parameters |
| 104 | `TreasuryLocked` | Treasury is locked | Wait for unlock period |

### Slashing Errors (120-139)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 120 | `ValidatorNotFound` | Validator not found | Register validator first |
| 121 | `InvalidSlashingAmount` | Invalid slashing amount | Use amount between 0.1-max |
| 122 | `SlashingAlreadyExecuted` | Slashing already executed | Can't execute twice |
| 123 | `SlashingPeriodNotActive` | Not in slashing period | Wait for slashing period |

### Risk Pool Errors (140-159)

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 140 | `RiskPoolNotFound` | Risk pool not found | Create pool first |
| 141 | `InvalidRiskPoolState` | Invalid risk pool state | Check pool status |
| 142 | `InsufficientRiskPoolBalance` | Insufficient risk pool balance | Deposit more funds |
| 143 | `RiskPoolLocked` | Risk pool is locked | Wait for unlock period |
| 144 | `InvalidReserveRatio` | Invalid reserve ratio | Use ratio between 20-100% |

## Error Code Allocation Rules

### When Adding New Errors

1. **Find appropriate range** - Match error category
2. **Use next available code** - No gaps or overlaps
3. **Update error messages** - Add to `.message()` method
4. **Update this document** - Keep reference current
5. **Update main docs** - Reflect in SHARED_TYPES_ERRORS_CONSTANTS.md

### Avoiding Conflicts

Check ranges:
- Get highest code in range
- Add 1 for new code
- Verify no duplicates across all files

```bash
# Check for duplicates
grep -r "= [0-9][0-9]$" contracts/*/src/
```

## Error Message Examples

Each error includes a human-readable message:

```rust
impl ContractError {
    pub fn message(&self) -> &str {
        match self {
            ContractError::InvalidCoverageAmount => "Coverage amount out of bounds",
            ContractError::InsufficientFunds => "Insufficient funds",
            // ... etc
        }
    }
}
```

Usage:
```rust
let error = ContractError::InvalidCoverageAmount;
println!("{}", error.message()); // "Coverage amount out of bounds"
```

## Error Conversion

Errors from other modules convert to ContractError:

```rust
impl From<AuthError> for ContractError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::Unauthorized => ContractError::Unauthorized,
            AuthError::InvalidRole => ContractError::InvalidRole,
            // ... etc
        }
    }
}
```

## Debugging Error Codes

### Finding Error Source

Each error code is unique, making it easy to find the source:

```bash
# Find error code 42
grep -r "= 42," contracts/
# Result: ClaimAmountExceedsCoverage
```

### Common Error Patterns

**Validation Error (code 3)**
- Input doesn't meet requirements
- Solution: Check input parameters

**Not Found Error (code 5)**
- Resource doesn't exist
- Solution: Use existing resource ID

**Insufficient Funds Error (code 4)**
- Balance too low
- Solution: Deposit or transfer funds

**Invalid State Error (code 7)**
- Operation not allowed in current state
- Solution: Check contract state first

## Statistics

- **Total Error Codes**: 62
- **Categories**: 8
- **Largest Category**: General (15)
- **Reserved Codes**: None
- **Future Growth**: Reserve 160-199+ if needed

## References

- [Shared Types, Errors & Constants Module](./SHARED_TYPES_ERRORS_CONSTANTS.md)
- [Shared Module Implementation](./SHARED_MODULE_IMPLEMENTATION.md)
- [Main README](./README.md)
