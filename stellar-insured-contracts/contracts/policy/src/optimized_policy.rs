//! Optimized policy contract implementation with gas efficiency improvements
//!
//! This module implements gas-optimized versions of policy operations:
//! - Compact storage representations
//! - Efficient state management
//! - Optimized query operations
//! - Reduced storage access patterns

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec, Map};
use insurance_contracts::gas_optimization::{GasOptimizer, OptimizedStructures, PerformanceMonitor};
use insurance_contracts::validation::{validate_coverage_amount, validate_premium_amount, validate_duration_days};

// Storage keys for optimized access
const POLICY_COUNTER_KEY: Symbol = Symbol::short("POL_CNT");
const ACTIVE_POLICIES_KEY: Symbol = Symbol::short("ACT_POL");
const POLICY_CONFIG_KEY: Symbol = Symbol::short("POL_CFG");
const POLICY_STATS_KEY: Symbol = Symbol::short("POL_STT");

/// Compact policy representation for reduced storage costs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactPolicy {
    /// Policy holder address (compressed)
    pub holder: Address,
    /// Coverage amount in compact format
    pub coverage: i64,  // Using i64 instead of i128 to save space
    /// Premium amount in compact format  
    pub premium: i64,   // Using i64 instead of i128 to save space
    /// Start time in days since epoch (u32 instead of u64)
    pub start_days: u32,
    /// Duration in days (u16 instead of u32)
    pub duration_days: u16,
    /// Compact state representation (1 byte)
    pub state: u8,
    /// Creation timestamp in days
    pub created_days: u32,
    /// Auto-renew flag packed with other flags
    pub flags: u8,
}

/// Policy statistics for efficient querying
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyStats {
    /// Total number of policies issued
    pub total_issued: u64,
    /// Number of active policies
    pub active_count: u32,
    /// Number of expired policies
    pub expired_count: u32,
    /// Number of cancelled policies
    pub cancelled_count: u32,
    /// Total coverage amount (in compact format)
    pub total_coverage: i64,
    /// Total premium collected (in compact format)
    pub total_premium: i64,
}

/// Configuration for optimized policy operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyConfig {
    /// Risk pool address
    pub risk_pool: Address,
    /// Minimum coverage amount (compact)
    pub min_coverage: i64,
    /// Maximum coverage amount (compact)
    pub max_coverage: i64,
    /// Minimum premium amount (compact)
    pub min_premium: i64,
    /// Maximum premium amount (compact)
    pub max_premium: i64,
    /// Batch size for efficient operations
    pub batch_size: u32,
}

/// Optimized policy contract implementation
pub struct OptimizedPolicyContract;

impl OptimizedPolicyContract {
    /// Convert standard policy to compact representation
    pub fn to_compact_policy(
        holder: Address,
        coverage_amount: i128,
        premium_amount: i128,
        start_time: u64,
        duration_days: u32,
        created_at: u64,
        auto_renew: bool,
    ) -> Result<CompactPolicy, crate::ContractError> {
        // Validate amounts fit in i64
        if coverage_amount > i64::MAX as i128 || coverage_amount < i64::MIN as i128 {
            return Err(crate::ContractError::InvalidAmount);
        }
        if premium_amount > i64::MAX as i128 || premium_amount < i64::MIN as i128 {
            return Err(crate::ContractError::InvalidPremium);
        }
        
        // Validate duration fits in u16
        if duration_days > u16::MAX as u32 {
            return Err(crate::ContractError::InvalidInput);
        }

        let flags = if auto_renew { 1u8 } else { 0u8 };
        
        Ok(CompactPolicy {
            holder,
            coverage: coverage_amount as i64,
            premium: premium_amount as i64,
            start_days: OptimizedStructures::timestamp_to_days(start_time),
            duration_days: duration_days as u16,
            state: OptimizedStructures::encode_policy_state(1), // ACTIVE = 1
            created_days: OptimizedStructures::timestamp_to_days(created_at),
            flags,
        })
    }

    /// Convert compact policy back to standard representation
    pub fn from_compact_policy(
        policy_id: u64,
        compact: &CompactPolicy,
    ) -> crate::Policy {
        crate::Policy {
            holder: compact.holder.clone(),
            coverage_amount: compact.coverage as i128,
            premium_amount: compact.premium as i128,
            start_time: OptimizedStructures::days_to_timestamp(compact.start_days),
            end_time: OptimizedStructures::days_to_timestamp(
                compact.start_days + compact.duration_days as u32
            ),
            state: OptimizedStructures::decode_policy_state(compact.state)
                .unwrap_or(crate::PolicyState::ACTIVE),
            created_at: OptimizedStructures::days_to_timestamp(compact.created_days),
            auto_renew: (compact.flags & 1) != 0,
        }
    }

    /// Get next policy ID using optimized counter storage
    pub fn next_policy_id_optimized(env: &Env) -> u64 {
        let current: u64 = GasOptimizer::cache_get(env, POLICY_COUNTER_KEY)
            .unwrap_or_else(|| {
                // Fall back to persistent storage
                env.storage().persistent().get(&POLICY_COUNTER_KEY).unwrap_or(0u64)
            });
        
        let next = current + 1;
        
        // Cache the updated value for fast access
        GasOptimizer::cache_set(env, POLICY_COUNTER_KEY, &next).unwrap();
        // Also update persistent storage
        env.storage().persistent().set(&POLICY_COUNTER_KEY, &next);
        
        next
    }

    /// Get or initialize policy statistics
    pub fn get_policy_stats(env: &Env) -> PolicyStats {
        env.storage().persistent().get(&POLICY_STATS_KEY).unwrap_or_else(|| {
            PolicyStats {
                total_issued: 0,
                active_count: 0,
                expired_count: 0,
                cancelled_count: 0,
                total_coverage: 0,
                total_premium: 0,
            }
        })
    }

    /// Update policy statistics efficiently
    pub fn update_policy_stats(
        env: &Env,
        old_state: Option<crate::PolicyState>,
        new_state: crate::PolicyState,
        coverage_change: i64,
        premium_change: i64,
    ) {
        let mut stats = Self::get_policy_stats(env);
        stats.total_issued += 1;
        
        // Update state counters
        if let Some(old) = old_state {
            match old {
                crate::PolicyState::ACTIVE => stats.active_count = stats.active_count.saturating_sub(1),
                crate::PolicyState::EXPIRED => stats.expired_count = stats.expired_count.saturating_sub(1),
                crate::PolicyState::CANCELLED => stats.cancelled_count = stats.cancelled_count.saturating_sub(1),
            }
        }
        
        match new_state {
            crate::PolicyState::ACTIVE => stats.active_count += 1,
            crate::PolicyState::EXPIRED => stats.expired_count += 1,
            crate::PolicyState::CANCELLED => stats.cancelled_count += 1,
        }
        
        // Update totals
        stats.total_coverage = stats.total_coverage.saturating_add(coverage_change);
        stats.total_premium = stats.total_premium.saturating_add(premium_change);
        
        env.storage().persistent().set(&POLICY_STATS_KEY, &stats);
    }

    /// Efficient batch policy query with pagination
    pub fn get_policies_batched(
        env: &Env,
        start_index: u32,
        limit: u32,
        filter_state: Option<crate::PolicyState>,
    ) -> (Vec<crate::PolicyView>, u32) {
        // Get active policy IDs from optimized storage
        let active_ids: Vec<u64> = env.storage().persistent().get(&ACTIVE_POLICIES_KEY)
            .unwrap_or_else(|| Vec::new(env));
        
        let total_count = active_ids.len() as u32;
        
        if start_index >= total_count {
            return (Vec::new(env), total_count);
        }
        
        let end_index = core::cmp::min(start_index + limit, total_count);
        let mut result = Vec::new(env);
        
        // Batch process policies
        for i in start_index..end_index {
            let policy_id = active_ids.get(i).unwrap();
            
            // Use instance storage for frequently accessed policies
            if let Some(compact_policy) = env.storage().instance().get::<_, CompactPolicy>(&Symbol::new(env, &format!("P_{}", policy_id))) {
                let policy = Self::from_compact_policy(policy_id, &compact_policy);
                
                // Apply state filter if specified
                if let Some(filter) = filter_state {
                    if policy.state() != filter {
                        continue;
                    }
                }
                
                let view = crate::PolicyView {
                    id: policy_id,
                    holder: policy.holder,
                    coverage_amount: policy.coverage_amount,
                    premium_amount: policy.premium_amount,
                    start_time: policy.start_time,
                    end_time: policy.end_time,
                    state: policy.state(),
                    created_at: policy.created_at,
                    auto_renew: policy.auto_renew,
                };
                result.push_back(view);
            }
        }
        
        (result, total_count)
    }

    /// Optimized policy issuance with reduced storage operations
    pub fn issue_policy_optimized(
        env: &Env,
        manager: Address,
        holder: Address,
        coverage_amount: i128,
        premium_amount: i128,
        duration_days: u32,
        auto_renew: bool,
    ) -> Result<u64, crate::ContractError> {
        // Validate inputs using shared validation
        validate_coverage_amount(coverage_amount)?;
        validate_premium_amount(premium_amount)?;
        validate_duration_days(duration_days)?;

        // Generate policy ID using optimized counter
        let policy_id = Self::next_policy_id_optimized(env);
        let current_time = env.ledger().timestamp();

        // Create compact policy representation
        let compact_policy = Self::to_compact_policy(
            holder.clone(),
            coverage_amount,
            premium_amount,
            current_time,
            duration_days,
            current_time,
            auto_renew,
        )?;

        // Store policy in instance storage for fast access (cheaper than persistent)
        env.storage().instance().set(&Symbol::new(env, &format!("P_{}", policy_id)), &compact_policy);

        // Update active policies list in persistent storage
        let mut active_list: Vec<u64> = env.storage().persistent().get(&ACTIVE_POLICIES_KEY)
            .unwrap_or_else(|| Vec::new(env));
        active_list.push_back(policy_id);
        env.storage().persistent().set(&ACTIVE_POLICIES_KEY, &active_list);

        // Update statistics
        Self::update_policy_stats(
            env,
            None,
            crate::PolicyState::ACTIVE,
            compact_policy.coverage,
            compact_policy.premium,
        );

        // Cache frequently accessed data
        GasOptimizer::cache_set(env, Symbol::new(env, &format!("HOLDER_{}", policy_id)), &holder)?;
        GasOptimizer::cache_set(env, Symbol::new(env, &format!("COV_{}", policy_id)), &coverage_amount)?;
        GasOptimizer::cache_set(env, Symbol::new(env, &format!("PREM_{}", policy_id)), &premium_amount)?;

        // Emit optimized event
        env.events().publish(
            (Symbol::new(env, "PolicyIssued"), policy_id),
            (holder, coverage_amount, premium_amount, duration_days as u64),
        );

        Ok(policy_id)
    }

    /// Optimized policy state transition with minimal storage writes
    pub fn transition_policy_state_optimized(
        env: &Env,
        policy_id: u64,
        target_state: crate::PolicyState,
    ) -> Result<(), crate::ContractError> {
        // Get policy from instance storage (fastest)
        let mut compact_policy: CompactPolicy = env.storage().instance()
            .get(&Symbol::new(env, &format!("P_{}", policy_id)))
            .ok_or(crate::ContractError::NotFound)?;

        let old_state = OptimizedStructures::decode_policy_state(compact_policy.state)
            .unwrap_or(crate::PolicyState::ACTIVE);

        // Validate state transition
        if !old_state.can_transition_to(target_state) {
            return Err(crate::ContractError::InvalidStateTransition);
        }

        // Update state in compact representation
        compact_policy.state = OptimizedStructures::encode_policy_state(match target_state {
            crate::PolicyState::ACTIVE => 1,
            crate::PolicyState::EXPIRED => 2,
            crate::PolicyState::CANCELLED => 3,
        });

        // Write back to instance storage only
        env.storage().instance().set(&Symbol::new(env, &format!("P_{}", policy_id)), &compact_policy);

        // Update active policies list if transitioning to terminal state
        if matches!(target_state, crate::PolicyState::CANCELLED | crate::PolicyState::EXPIRED) {
            let mut active_list: Vec<u64> = env.storage().persistent().get(&ACTIVE_POLICIES_KEY)
                .unwrap_or_else(|| Vec::new(env));
            
            // Remove policy ID from active list (optimized removal)
            let mut new_list = Vec::new(env);
            for i in 0..active_list.len() {
                let id = active_list.get(i).unwrap();
                if id != policy_id {
                    new_list.push_back(id);
                }
            }
            env.storage().persistent().set(&ACTIVE_POLICIES_KEY, &new_list);
        }

        // Update statistics efficiently
        Self::update_policy_stats(
            env,
            Some(old_state),
            target_state,
            0, // No coverage change
            0, // No premium change
        );

        // Clear cached data for this policy
        GasOptimizer::cache_clear(env, Symbol::new(env, &format!("HOLDER_{}", policy_id)));
        GasOptimizer::cache_clear(env, Symbol::new(env, &format!("COV_{}", policy_id)));
        GasOptimizer::cache_clear(env, Symbol::new(env, &format!("PREM_{}", policy_id)));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Address};

    #[test]
    fn test_compact_policy_conversion() {
        let env = Env::default();
        let holder = Address::generate(&env);
        
        let coverage = 1_000_000_000i128; // 1000 XLM
        let premium = 10_000_000i128;     // 10 XLM
        let start_time = 1704067200u64;   // Jan 1, 2024
        let duration = 30u32;
        let created_at = start_time;
        
        let compact = OptimizedPolicyContract::to_compact_policy(
            holder.clone(),
            coverage,
            premium,
            start_time,
            duration,
            created_at,
            true,
        ).unwrap();
        
        assert_eq!(compact.holder, holder);
        assert_eq!(compact.coverage, coverage as i64);
        assert_eq!(compact.premium, premium as i64);
        assert_eq!(compact.duration_days, duration as u16);
        assert_eq!(compact.state, 1); // ACTIVE
        assert_eq!(compact.flags, 1); // auto_renew = true
        
        let standard = OptimizedPolicyContract::from_compact_policy(1, &compact);
        assert_eq!(standard.holder, holder);
        assert_eq!(standard.coverage_amount, coverage);
        assert_eq!(standard.premium_amount, premium);
        assert_eq!(standard.state(), crate::PolicyState::ACTIVE);
        assert_eq!(standard.auto_renew, true);
    }

    #[test]
    fn test_policy_id_optimization() {
        let env = Env::default();
        
        // First ID should be 1
        let id1 = OptimizedPolicyContract::next_policy_id_optimized(&env);
        assert_eq!(id1, 1);
        
        // Second ID should be 2
        let id2 = OptimizedPolicyContract::next_policy_id_optimized(&env);
        assert_eq!(id2, 2);
        
        // Value should be cached
        let cached: u64 = GasOptimizer::cache_get(&env, POLICY_COUNTER_KEY).unwrap();
        assert_eq!(cached, 2);
    }

    #[test]
    fn test_policy_stats() {
        let env = Env::default();
        
        // Initial stats should be zeros
        let stats = OptimizedPolicyContract::get_policy_stats(&env);
        assert_eq!(stats.total_issued, 0);
        assert_eq!(stats.active_count, 0);
        
        // Update stats for new policy
        OptimizedPolicyContract::update_policy_stats(
            &env,
            None,
            crate::PolicyState::ACTIVE,
            1000,
            100,
        );
        
        let updated_stats = OptimizedPolicyContract::get_policy_stats(&env);
        assert_eq!(updated_stats.total_issued, 1);
        assert_eq!(updated_stats.active_count, 1);
        assert_eq!(updated_stats.total_coverage, 1000);
        assert_eq!(updated_stats.total_premium, 100);
    }

    #[test]
    fn test_compact_timestamp_storage() {
        let timestamp = 1704067200u64; // Jan 1, 2024
        let days = OptimizedStructures::timestamp_to_days(timestamp);
        assert_eq!(days, 19723); // Days since Stellar epoch
        
        let reconstructed = OptimizedStructures::days_to_timestamp(days);
        // Should be within 1 day precision (86400 seconds)
        assert!(reconstructed.abs_diff(timestamp) <= 86400);
    }
}