//! Common error types for insurance contracts
//!
//! This module defines a unified set of error codes that are used across all
//! insurance contracts to ensure consistent error handling and reporting.

use soroban_sdk::contracterror;

/// Comprehensive error type for insurance contracts
///
/// All errors are assigned unique codes for easy identification and debugging.
/// Error ranges are organized by category:
/// - 1-19: General/Authorization errors
/// - 20-39: Policy-specific errors
/// - 40-59: Claim-specific errors
/// - 60-79: Oracle-specific errors
/// - 80-99: Governance errors
/// - 100-119: Treasury errors
/// - 120-139: Slashing errors
/// - 140-159: Risk Pool errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    // ===== General/Authorization Errors (1-19) =====
    /// Caller is not authorized to perform this action
    Unauthorized = 1,

    /// Contract is paused and operations are not allowed
    Paused = 2,

    /// Invalid input provided
    InvalidInput = 3,

    /// Insufficient funds for operation
    InsufficientFunds = 4,

    /// Requested resource not found
    NotFound = 5,

    /// Resource already exists
    AlreadyExists = 6,

    /// Invalid state for operation
    InvalidState = 7,

    /// Arithmetic overflow occurred
    Overflow = 8,

    /// Contract not initialized
    NotInitialized = 9,

    /// Contract already initialized
    AlreadyInitialized = 10,

    /// Invalid role or permission
    InvalidRole = 11,

    /// Role not found
    RoleNotFound = 12,

    /// Contract not trusted for cross-contract calls
    NotTrustedContract = 13,

    /// Invalid address format or value
    InvalidAddress = 14,

    /// Operation would cause underflow
    Underflow = 15,

    // ===== Policy-Specific Errors (20-39) =====
    /// Policy not found
    PolicyNotFound = 20,

    /// Invalid policy state for operation
    InvalidPolicyState = 21,

    /// Coverage amount out of allowed bounds
    InvalidCoverageAmount = 22,

    /// Premium amount out of allowed bounds
    InvalidPremiumAmount = 23,

    /// Policy duration out of allowed bounds
    InvalidDuration = 24,

    /// Cannot renew an expired or cancelled policy
    CannotRenewPolicy = 25,

    /// State transition is not allowed
    InvalidStateTransition = 26,

    // ===== Claim-Specific Errors (40-59) =====
    /// Claim not found
    ClaimNotFound = 40,

    /// Invalid claim state for operation
    InvalidClaimState = 41,

    /// Claim amount exceeds coverage
    ClaimAmountExceedsCoverage = 42,

    /// Claim period has expired
    ClaimPeriodExpired = 43,

    /// Cannot submit claim for this policy
    CannotSubmitClaim = 44,

    /// Policy coverage has expired
    PolicyCoverageExpired = 45,

    /// Evidence-related error
    EvidenceError = 46,

    /// Evidence already exists
    EvidenceAlreadyExists = 47,

    /// Evidence not found
    EvidenceNotFound = 48,

    /// Invalid evidence hash
    InvalidEvidenceHash = 49,

    // ===== Oracle-Specific Errors (60-79) =====
    /// Oracle validation failed
    OracleValidationFailed = 60,

    /// Insufficient oracle submissions
    InsufficientOracleSubmissions = 61,

    /// Oracle data is stale
    OracleDataStale = 62,

    /// Oracle data is an outlier
    OracleOutlierDetected = 63,

    /// Oracle contract not configured
    OracleNotConfigured = 64,

    /// Oracle contract is invalid
    InvalidOracleContract = 65,

    // ===== Governance Errors (80-99) =====
    /// Voting period has ended
    VotingPeriodEnded = 80,

    /// Address has already voted
    AlreadyVoted = 81,

    /// Proposal not active
    ProposalNotActive = 82,

    /// Quorum not met
    QuorumNotMet = 83,

    /// Threshold not met
    ThresholdNotMet = 84,

    /// Proposal not found
    ProposalNotFound = 85,

    /// Invalid proposal type
    InvalidProposalType = 86,

    /// Slashing contract not set
    SlashingContractNotSet = 87,

    /// Slashing execution failed
    SlashingExecutionFailed = 88,

    // ===== Treasury Errors (100-119) =====
    /// Treasury fund not found
    TreasuryFundNotFound = 100,

    /// Insufficient treasury balance
    InsufficientTreasuryBalance = 101,

    /// Invalid allocation
    InvalidAllocation = 102,

    /// Invalid distribution
    InvalidDistribution = 103,

    /// Treasury locked
    TreasuryLocked = 104,

    // ===== Slashing Errors (120-139) =====
    /// Validator not found
    ValidatorNotFound = 120,

    /// Invalid slashing amount
    InvalidSlashingAmount = 121,

    /// Slashing already executed
    SlashingAlreadyExecuted = 122,

    /// Slashing period not active
    SlashingPeriodNotActive = 123,

    // ===== Risk Pool Errors (140-159) =====
    /// Risk pool not found
    RiskPoolNotFound = 140,

    /// Invalid risk pool state
    InvalidRiskPoolState = 141,

    /// Insufficient risk pool balance
    InsufficientRiskPoolBalance = 142,

    /// Risk pool locked
    RiskPoolLocked = 143,

    /// Invalid reserve ratio
    InvalidReserveRatio = 144,
}

/// Detailed error message provider
impl ContractError {
    /// Get a human-readable description of the error
    pub fn message(&self) -> &str {
        match self {
            // General/Authorization
            ContractError::Unauthorized => "Caller is not authorized",
            ContractError::Paused => "Contract is paused",
            ContractError::InvalidInput => "Invalid input provided",
            ContractError::InsufficientFunds => "Insufficient funds",
            ContractError::NotFound => "Resource not found",
            ContractError::AlreadyExists => "Resource already exists",
            ContractError::InvalidState => "Invalid state for operation",
            ContractError::Overflow => "Arithmetic overflow",
            ContractError::NotInitialized => "Contract not initialized",
            ContractError::AlreadyInitialized => "Contract already initialized",
            ContractError::InvalidRole => "Invalid role",
            ContractError::RoleNotFound => "Role not found",
            ContractError::NotTrustedContract => "Contract not trusted",
            ContractError::InvalidAddress => "Invalid address",
            ContractError::Underflow => "Arithmetic underflow",

            // Policy-Specific
            ContractError::PolicyNotFound => "Policy not found",
            ContractError::InvalidPolicyState => "Invalid policy state",
            ContractError::InvalidCoverageAmount => "Invalid coverage amount",
            ContractError::InvalidPremiumAmount => "Invalid premium amount",
            ContractError::InvalidDuration => "Invalid policy duration",
            ContractError::CannotRenewPolicy => "Cannot renew this policy",
            ContractError::InvalidStateTransition => "Invalid state transition",

            // Claim-Specific
            ContractError::ClaimNotFound => "Claim not found",
            ContractError::InvalidClaimState => "Invalid claim state",
            ContractError::ClaimAmountExceedsCoverage => "Claim exceeds coverage",
            ContractError::ClaimPeriodExpired => "Claim period expired",
            ContractError::CannotSubmitClaim => "Cannot submit claim for this policy",
            ContractError::PolicyCoverageExpired => "Policy coverage has expired",
            ContractError::EvidenceError => "Evidence error",
            ContractError::EvidenceAlreadyExists => "Evidence already exists",
            ContractError::EvidenceNotFound => "Evidence not found",
            ContractError::InvalidEvidenceHash => "Invalid evidence hash",

            // Oracle-Specific
            ContractError::OracleValidationFailed => "Oracle validation failed",
            ContractError::InsufficientOracleSubmissions => "Insufficient oracle submissions",
            ContractError::OracleDataStale => "Oracle data is stale",
            ContractError::OracleOutlierDetected => "Oracle data is an outlier",
            ContractError::OracleNotConfigured => "Oracle not configured",
            ContractError::InvalidOracleContract => "Invalid oracle contract",

            // Governance
            ContractError::VotingPeriodEnded => "Voting period has ended",
            ContractError::AlreadyVoted => "Already voted on this proposal",
            ContractError::ProposalNotActive => "Proposal is not active",
            ContractError::QuorumNotMet => "Quorum not met",
            ContractError::ThresholdNotMet => "Threshold not met",
            ContractError::ProposalNotFound => "Proposal not found",
            ContractError::InvalidProposalType => "Invalid proposal type",
            ContractError::SlashingContractNotSet => "Slashing contract not set",
            ContractError::SlashingExecutionFailed => "Slashing execution failed",

            // Treasury
            ContractError::TreasuryFundNotFound => "Treasury fund not found",
            ContractError::InsufficientTreasuryBalance => "Insufficient treasury balance",
            ContractError::InvalidAllocation => "Invalid allocation",
            ContractError::InvalidDistribution => "Invalid distribution",
            ContractError::TreasuryLocked => "Treasury is locked",

            // Slashing
            ContractError::ValidatorNotFound => "Validator not found",
            ContractError::InvalidSlashingAmount => "Invalid slashing amount",
            ContractError::SlashingAlreadyExecuted => "Slashing already executed",
            ContractError::SlashingPeriodNotActive => "Slashing period not active",

            // Risk Pool
            ContractError::RiskPoolNotFound => "Risk pool not found",
            ContractError::InvalidRiskPoolState => "Invalid risk pool state",
            ContractError::InsufficientRiskPoolBalance => "Insufficient risk pool balance",
            ContractError::RiskPoolLocked => "Risk pool is locked",
            ContractError::InvalidReserveRatio => "Invalid reserve ratio",
        }
    }
}
