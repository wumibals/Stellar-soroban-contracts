#![allow(unused)]

use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum PolicyType {
    Weather,
    Crop,
    Flight,
    Health,
    Property,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum PolicyStatus {
    Active,
    Expired,
    Claimed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum TriggerType {
    WeatherIndex,
    PriceIndex,
    FlightDelay,
    HealthEvent,
    CustomOracle,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InsurancePolicy {
    pub id: u64,
    pub policyholder: Address,
    pub coverage_amount: i128,
    pub premium_paid: i128,
    pub policy_type: PolicyType,
    pub start_time: u64,
    pub end_time: u64,
    pub status: PolicyStatus,
    pub tokens_minted: i128,
    pub trigger_id: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Listing {
    pub id: u64,
    pub seller: Address,
    pub policy_id: u64,
    pub amount: i128,
    pub price_per_token: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ParametricTrigger {
    pub id: u64,
    pub trigger_type: TriggerType,
    pub threshold: i128,
    pub policy_type: PolicyType,
    pub is_triggered: bool,
    pub last_value: i128,
    pub is_above_threshold: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Claim {
    pub id: u64,
    pub policy_id: u64,
    pub claimant: Address,
    pub amount: i128,
    pub is_approved: bool,
    pub is_paid: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityPool {
    pub insurance_reserve: i128,
    pub base_reserve: i128,
    pub total_lp_tokens: i128,
    pub fee_rate: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AdminConfig {
    pub admin: Address,
    pub oracle: Address,
    pub base_token: Address,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    PolicyCount,
    Policy(u64),
    ListingCount,
    Listing(u64),
    TriggerCount,
    Trigger(u64),
    ClaimCount,
    Claim(u64),
    Pool,
    Balance(Address, u64),
    Allowance(Address, Address, u64),
    TotalSupply(u64),
    LpBalance(Address),
}
