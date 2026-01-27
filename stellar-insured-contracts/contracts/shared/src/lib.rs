#![no_std]
//! # Shared Insurance Contracts Library
//!
//! A comprehensive library providing reusable types, errors, constants, and validation
//! helpers for all Stellar Insured Soroban contracts.
//!
//! ## Modules
//!
//! - `errors` - Common error types used across contracts
//! - `types` - Shared data types and enums (PolicyStatus, ClaimStatus, etc.)
//! - `constants` - Configuration constants for validation and limits
//! - `validation` - Reusable validation helper functions
//!
//! ## Usage
//!
//! Import the shared library in your contract's Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! shared = { path = "shared" }
//! ```
//!
//! Then use it in your code:
//!
//! ```rust,ignore
//! use shared::errors::ContractError;
//! use shared::types::{PolicyStatus, ClaimStatus};
//! use shared::validation::validate_address;
//! use shared::constants::MIN_COVERAGE_AMOUNT;
//! ```

pub mod errors;
pub mod types;
pub mod constants;
pub mod validation;

// Re-export commonly used types
pub use errors::ContractError;
pub use types::{
    PolicyStatus, ClaimStatus, ProposalStatus, ProposalType, VoteType,
    RiskPoolStatus, ClaimEvidence, VoteRecord, OracleConfig, RiskMetrics,
    PolicyMetadata, ClaimMetadata, TreasuryAllocation, DataKey,
};
pub use validation::{
    validate_address, validate_positive_amount, validate_coverage_amount,
    validate_premium_amount, validate_duration_days, validate_not_paused,
    safe_add, safe_sub, safe_mul, safe_div, calculate_percentage,
};
