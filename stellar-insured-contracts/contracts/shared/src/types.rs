//! Common types and data structures for insurance contracts
//!
//! This module defines shared enums and structs that represent core concepts
//! used across all insurance contracts (policies, claims, governance, etc.).

use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

// ===== Asset Types =====

/// Represents an asset type in the insurance protocol
/// Supports both native XLM and custom Stellar assets
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    /// Native XLM asset
    Native,
    /// Stellar asset with issuer (asset_code, issuer_address)
    Stellar((Symbol, Address)),
    /// Contract-based token (token contract address)
    Contract(Address),
}

impl Asset {
    /// Returns a unique identifier for the asset for storage purposes
    pub fn to_key(&self) -> Symbol {
        match self {
            Asset::Native => Symbol::new(&soroban_sdk::Env::default(), "XLM"),
            Asset::Stellar((code, _)) => code.clone(),
            Asset::Contract(_addr) => {
                // Use first 4 bytes of address as identifier
                Symbol::new(&soroban_sdk::Env::default(), "CONTR")
            }
        }
    }
}

/// Asset metadata for registered assets
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetMetadata {
    /// The asset identifier
    pub asset: Asset,
    /// Asset symbol (e.g., "USDC", "XLM")
    pub symbol: Symbol,
    /// Asset name
    pub name: Symbol,
    /// Number of decimal places
    pub decimals: u32,
    /// Whether the asset is active for use
    pub is_active: bool,
    /// Whether the asset is accepted for premiums
    pub accept_for_premium: bool,
    /// Whether the asset is accepted for claims
    pub accept_for_claims: bool,
    /// Minimum amount allowed for transactions
    pub min_amount: i128,
    /// Maximum amount allowed for transactions
    pub max_amount: i128,
    /// Timestamp when asset was registered
    pub registered_at: u64,
}

/// Multi-asset balance structure for tracking balances across assets
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiAssetBalance {
    /// The asset type
    pub asset: Asset,
    /// Balance amount
    pub amount: i128,
    /// Last updated timestamp
    pub updated_at: u64,
}

/// Asset conversion rate information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetConversionRate {
    /// Source asset
    pub from_asset: Asset,
    /// Target asset
    pub to_asset: Asset,
    /// Conversion rate in basis points (e.g., 10000 = 1:1)
    /// Rate represents (to_amount * 10000) / from_amount
    pub rate_bps: u32,
    /// Timestamp when rate was last updated
    pub updated_at: u64,
    /// Oracle source address
    pub oracle_source: Address,
}

/// Policy asset configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyAssetConfig {
    /// Asset used for coverage amount
    pub coverage_asset: Asset,
    /// Asset used for premium payments
    pub premium_asset: Asset,
    /// Whether multi-asset claims are allowed
    pub allow_multi_asset_claims: bool,
    /// List of assets accepted for claim payouts (if multi-asset enabled)
    pub accepted_claim_assets: Vec<Asset>,
}

/// Claim payout asset preference
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimPayoutPreference {
    /// Preferred asset for payout
    pub preferred_asset: Asset,
    /// Whether to accept equivalent value in other assets
    pub accept_alternative: bool,
    /// Alternative assets accepted (in order of preference)
    pub alternatives: Vec<Asset>,
}

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

// ===== Product Template Types =====

/// Insurance product categories
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductCategory {
    /// Property insurance (homes, buildings, etc.)
    Property = 0,
    /// Health insurance
    Health = 1,
    /// Auto insurance
    Auto = 2,
    /// Life insurance
    Life = 3,
    /// Travel insurance
    Travel = 4,
    /// Cyber insurance
    Cyber = 5,
    /// Business insurance
    Business = 6,
    /// Custom/Other insurance
    Custom = 7,
}

/// Template status lifecycle
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateStatus {
    /// Template is being drafted
    Draft = 0,
    /// Template submitted for review
    PendingReview = 1,
    /// Template approved and ready for use
    Approved = 2,
    /// Template is active and can be used to create policies
    Active = 3,
    /// Template is deprecated but existing policies remain valid
    Deprecated = 4,
    /// Template is archived and cannot be used
    Archived = 5,
}

/// Risk level classification
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    /// Low risk - minimal likelihood of claims
    Low = 0,
    /// Medium risk - moderate likelihood of claims
    Medium = 1,
    /// High risk - significant likelihood of claims
    High = 2,
    /// Very high risk - maximum likelihood of claims
    VeryHigh = 3,
}

/// Premium calculation model
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PremiumModel {
    /// Fixed premium amount
    Fixed = 0,
    /// Percentage of coverage amount
    Percentage = 1,
    /// Risk-based calculation
    RiskBased = 2,
    /// Tiered pricing based on coverage tiers
    Tiered = 3,
}

/// Coverage type specification
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoverageType {
    /// Full coverage for specified risks
    Full = 0,
    /// Partial coverage with specified limits
    Partial = 1,
    /// Excess coverage above deductible
    Excess = 2,
}

/// Customization parameter types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomParam {
    /// Integer parameter (e.g., coverage limit, duration)
    Integer((Symbol, i128, i128, i128)),
    /// Decimal parameter (e.g., premium rate, deductible percentage)
    Decimal((Symbol, i128, i128, i128)),
    /// Boolean parameter (e.g., additional coverage options)
    Boolean((Symbol, bool)),
    /// Choice parameter from predefined options
    Choice((Symbol, Vec<Symbol>, u32)),
}

/// Product template definition
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductTemplate {
    /// Unique template identifier
    pub id: u64,
    /// Template name
    pub name: Symbol,
    /// Template description
    pub description: Symbol,
    /// Product category
    pub category: ProductCategory,
    /// Current template status
    pub status: TemplateStatus,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Premium calculation model
    pub premium_model: PremiumModel,
    /// Coverage type
    pub coverage_type: CoverageType,
    /// Minimum coverage amount allowed
    pub min_coverage: i128,
    /// Maximum coverage amount allowed
    pub max_coverage: i128,
    /// Minimum policy duration in days
    pub min_duration_days: u32,
    /// Maximum policy duration in days
    pub max_duration_days: u32,
    /// Base premium rate (basis points, 0-10000)
    pub base_premium_rate_bps: u32,
    /// Minimum deductible amount
    pub min_deductible: i128,
    /// Maximum deductible amount
    pub max_deductible: i128,
    /// Required collateral ratio (basis points, 0-10000)
    pub collateral_ratio_bps: u32,
    /// Customizable parameters
    pub custom_params: Vec<CustomParam>,
    /// Creator/administrator address
    pub creator: Address,
    /// Timestamp when template was created
    pub created_at: u64,
    /// Timestamp of last update
    pub updated_at: u64,
    /// Version number for template updates
    pub version: u32,
}

/// Custom parameter values for a specific policy instance
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomParamValue {
    /// Parameter name
    pub name: Symbol,
    /// Parameter value
    pub value: CustomParamValueData,
}

/// Custom parameter value data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomParamValueData {
    /// Integer value
    Integer(i128),
    /// Decimal value
    Decimal(i128),
    /// Boolean value
    Boolean(bool),
    /// Choice index
    Choice(u32),
}

/// Policy instance created from a template
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplatePolicy {
    /// Policy identifier
    pub policy_id: u64,
    /// Template ID this policy was created from
    pub template_id: u64,
    /// Policy holder address
    pub holder: Address,
    /// Selected coverage amount
    pub coverage_amount: i128,
    /// Calculated premium amount
    pub premium_amount: i128,
    /// Policy duration in days
    pub duration_days: u32,
    /// Selected deductible amount
    pub deductible: i128,
    /// Custom parameter values
    pub custom_values: Vec<CustomParamValue>,
    /// Timestamp when policy was created
    pub created_at: u64,
    /// Timestamp when policy starts
    pub start_time: u64,
    /// Timestamp when policy expires
    pub end_time: u64,
}

/// Template validation rules
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateValidationRules {
    /// Minimum required collateral ratio (basis points)
    pub min_collateral_ratio_bps: u32,
    /// Maximum allowed premium rate (basis points)
    pub max_premium_rate_bps: u32,
    /// Minimum policy duration
    pub min_duration_days: u32,
    /// Maximum policy duration
    pub max_duration_days: u32,
    /// Required governance approval threshold for new templates
    pub approval_threshold_bps: u32,
    /// Minimum time between template updates (seconds)
    pub min_update_interval: u64,
}

// ===== Cross-Chain Types =====

/// Status of a cross-chain message
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChainMessageStatus {
    /// Message has been sent/queued
    Pending = 0,
    /// Message has been confirmed by validators
    Confirmed = 1,
    /// Message has been executed on target chain
    Executed = 2,
    /// Message has failed execution
    Failed = 3,
    /// Message has expired
    Expired = 4,
}

/// Types of cross-chain messages
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChainMessageType {
    /// Asset transfer between chains
    AssetTransfer = 0,
    /// Governance action across chains
    GovernanceAction = 1,
    /// Data synchronization between chains
    DataSync = 2,
    /// Insurance claim across chains
    InsuranceClaim = 3,
    /// Policy update across chains
    PolicyUpdate = 4,
}

/// Status of a registered bridge
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeStatus {
    /// Bridge is active and operational
    Active = 0,
    /// Bridge is temporarily paused
    Paused = 1,
    /// Bridge has been deprecated
    Deprecated = 2,
    /// Bridge has been deactivated
    Inactive = 3,
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
    /// Product template by ID
    ProductTemplate,
    /// Template counter
    TemplateCounter,
    /// Template policy by ID
    TemplatePolicy,
    /// Template policy counter
    TemplatePolicyCounter,
    /// Template validation rules
    TemplateValidationRules,
    /// Template status history
    TemplateStatusHistory,
    /// Cross-chain bridge registration
    CrossChainBridge,
    /// Cross-chain message
    CrossChainMessage,
    /// Cross-chain asset mapping
    CrossChainAssetMap,
    /// Cross-chain validator
    CrossChainValidator,
    /// Cross-chain governance proposal
    CrossChainProposal,
    /// Cross-chain counter
    CrossChainCounter,
    /// Asset registry storage
    AssetRegistry,
    /// Asset metadata by asset
    AssetMetadata,
    /// Asset conversion rate
    AssetConversionRate,
    /// Multi-asset balance
    MultiAssetBalance,
    /// Policy asset configuration
    PolicyAssetConfig,
    /// Asset balance by (owner, asset)
    AssetBalance,
}
