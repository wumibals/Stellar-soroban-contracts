//! Common types and data structures for insurance contracts
//!
//! This module defines shared enums and structs that represent core concepts
//! used across all insurance contracts (policies, claims, governance, etc.).

use soroban_sdk::{contracttype, Address, BytesN};

// ===== Status Enums =====

/// Represents the lifecycle status of a policy
///
/// # Transitions
/// - `Active` → `Expired` or `Cancelled`
/// - `Expired` → Terminal (no further transitions)
/// - `Cancelled` → Terminal (no further transitions)
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyStatus {
    /// Policy is currently active and valid
    Active = 0,

    /// Policy has expired naturally
    Expired = 1,

    /// Policy has been cancelled
    Cancelled = 2,

    /// Policy has been claimed against
    Claimed = 3,
}

/// Represents the lifecycle status of a claim
///
/// # Transitions
/// - `Submitted` → `UnderReview`, `Rejected`
/// - `UnderReview` → `Approved`, `Rejected`
/// - `Approved` → `Settled`
/// - `Rejected` → Terminal (no further transitions)
/// - `Settled` → Terminal (no further transitions)
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    /// Claim has been submitted but not reviewed
    Submitted = 0,

    /// Claim is currently under review
    UnderReview = 1,

    /// Claim has been approved
    Approved = 2,

    /// Claim has been rejected
    Rejected = 3,

    /// Claim has been settled (payment made)
    Settled = 4,
}

/// Represents the status of a governance proposal
///
/// # Transitions
/// - `Active` → `Passed`, `Rejected`, or `Expired`
/// - `Passed` → `Executed`
/// - `Rejected` → Terminal (no further transitions)
/// - `Executed` → Terminal (no further transitions)
/// - `Expired` → Terminal (no further transitions)
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is currently active for voting
    Active = 0,

    /// Proposal has passed voting
    Passed = 1,

    /// Proposal has been rejected
    Rejected = 2,

    /// Proposal has been executed
    Executed = 3,

    /// Proposal voting period has expired
    Expired = 4,
}

/// Types of governance proposals
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalType {
    /// Proposal to change protocol parameters
    ParameterChange = 0,

    /// Proposal to upgrade a contract
    ContractUpgrade = 1,

    /// Proposal to execute slashing
    SlashingAction = 2,

    /// Proposal to allocate treasury funds
    TreasuryAllocation = 3,

    /// Proposal for emergency actions
    EmergencyAction = 4,
}

/// Vote choice in governance
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoteType {
    /// Vote in favor
    Yes = 0,

    /// Vote against
    No = 1,

    /// Abstain from voting
    Abstain = 2,
}

/// Risk pool status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskPoolStatus {
    /// Risk pool is operational
    Active = 0,

    /// Risk pool is temporarily closed
    Paused = 1,

    /// Risk pool is in emergency mode
    Emergency = 2,

    /// Risk pool is shut down
    Closed = 3,
}

// ===== Data Structures =====

/// Represents evidence for a claim (hash-only, immutable)
///
/// Evidence is stored as a SHA-256 hash to maintain immutability
/// while keeping storage costs reasonable.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimEvidence {
    /// Unique claim identifier
    pub claim_id: BytesN<32>,

    /// SHA-256 hash of the evidence
    pub evidence_hash: BytesN<32>,

    /// Address that submitted the evidence
    pub submitter: Address,

    /// Timestamp when evidence was submitted
    pub submitted_at: u64,
}

/// Represents a vote record in governance
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteRecord {
    /// Proposal identifier
    pub proposal_id: u64,

    /// Address of the voter
    pub voter: Address,

    /// Vote choice (Yes, No, Abstain)
    pub vote: VoteType,

    /// Voting power used
    pub voting_power: i128,

    /// Timestamp of the vote
    pub voted_at: u64,
}

/// Configuration for oracle validation
///
/// Used to configure how claims and other operations are validated
/// against oracle data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConfig {
    /// Address of the oracle contract
    pub oracle_contract: Address,

    /// Whether oracle validation is required
    pub require_oracle_validation: bool,

    /// Minimum number of oracle submissions required
    pub min_oracle_submissions: u32,

    /// Maximum allowed age of oracle data in seconds
    pub max_data_age: u64,

    /// Maximum allowed deviation from median (basis points, 0-10000)
    pub max_deviation_bps: u32,
}

/// Risk metrics for a policy or pool
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskMetrics {
    /// Total value at risk
    pub total_value_at_risk: i128,

    /// Current reserve balance
    pub reserve_balance: i128,

    /// Reserve ratio as percentage (0-100)
    pub reserve_ratio_percent: u32,

    /// Total claims paid out
    pub total_claims_paid: i128,

    /// Loss ratio percentage (0-100)
    pub loss_ratio_percent: u32,
}

/// Policy metadata for tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyMetadata {
    /// Policy identifier
    pub policy_id: u64,

    /// Policy holder address
    pub holder: Address,

    /// Coverage amount in stroops
    pub coverage_amount: i128,

    /// Premium amount in stroops
    pub premium_amount: i128,

    /// Policy start timestamp
    pub start_time: u64,

    /// Policy end timestamp
    pub end_time: u64,

    /// Policy status
    pub status: PolicyStatus,

    /// Timestamp when policy was created
    pub created_at: u64,

    /// Timestamp of last update
    pub updated_at: u64,
}

/// Claim metadata for tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimMetadata {
    /// Claim identifier
    pub claim_id: u64,

    /// Associated policy identifier
    pub policy_id: u64,

    /// Claimant address
    pub claimant: Address,

    /// Claimed amount in stroops
    pub claimed_amount: i128,

    /// Approved amount in stroops
    pub approved_amount: i128,

    /// Claim status
    pub status: ClaimStatus,

    /// Timestamp when claim was submitted
    pub submitted_at: u64,

    /// Timestamp of last update
    pub updated_at: u64,

    /// Optional evidence hash (SHA-256)
    pub evidence_hash: Option<BytesN<32>>,
}

/// Treasury allocation record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryAllocation {
    /// Allocation identifier
    pub allocation_id: u64,

    /// Recipient address
    pub recipient: Address,

    /// Allocated amount
    pub amount: i128,

    /// Purpose of allocation
    pub purpose: BytesN<32>, // Hash of purpose string

    /// Timestamp when allocated
    pub allocated_at: u64,

    /// Timestamp when funds were released
    pub released_at: Option<u64>,

    /// Whether allocation has been executed
    pub executed: bool,
}

// ===== Common Enums for Storage Keys =====

/// Data key enumeration for contract storage
///
/// These are used to organize data in contract storage in a type-safe way.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Admin address key
    Admin,

    /// Paused state key
    Paused,

    /// General configuration key
    Config,

    /// Counter for various entities (Symbol → counter value)
    Counter,

    /// Policy by ID
    Policy,

    /// Claim by ID
    Claim,

    /// Governance proposal by ID
    Proposal,

    /// Claim evidence by claim ID
    ClaimEvidence,

    /// Oracle configuration
    OracleConfig,

    /// Treasury fund
    Treasury,

    /// Risk pool state
    RiskPool,

    /// Validator information
    Validator,

    /// Slashing record
    SlashingRecord,

    /// Authorization role
    AuthRole,
}
