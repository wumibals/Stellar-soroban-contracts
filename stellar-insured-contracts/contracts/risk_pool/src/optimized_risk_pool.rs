//! Optimized risk pool contract with gas efficiency improvements
//!
//! This module implements gas-optimized versions of risk pool operations:
//! - Efficient arithmetic operations
//! - Optimized storage access patterns
//! - Compact data representations
//! - Batch operations for reduced storage costs

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec, Map};
use insurance_contracts::gas_optimization::{GasOptimizer, OptimizedStructures, PerformanceMonitor};

/// Compact risk pool statistics for reduced storage costs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactPoolStats {
    /// Total liquidity (compact format using i64)
    pub total_liquidity: i64,
    /// Total claims paid (compact format)
    pub total_claims_paid: i64,
    /// Total deposits (compact format)
    pub total_deposits: i64,
    /// Provider count (u32 instead of u64)
    pub provider_count: u32,
    /// Last update timestamp in days
    pub last_update_days: u32,
}

/// Compact provider information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactProviderInfo {
    /// Current balance (compact)
    pub balance: i64,
    /// Total deposited (compact)
    pub total_deposited: i64,
    /// Registration timestamp in days
    pub registered_days: u32,
    /// Flags for various states (bit-packed)
    pub flags: u8,
}

/// Storage optimization keys
const POOL_STATS_KEY: Symbol = Symbol::short("POOL_ST");
const RESERVED_TOTAL_KEY: Symbol = Symbol::short("RSV_TOT");
const PROVIDER_COUNT_KEY: Symbol = Symbol::short("PROV_CNT");
const CONFIG_KEY: Symbol = Symbol::short("CFG");

/// Configuration constants for optimization
const BATCH_SIZE: u32 = 50; // Optimal batch size for storage operations
const CACHE_TTL_DAYS: u32 = 7; // Cache time-to-live in days

/// Optimized risk pool operations
pub struct OptimizedRiskPool;

impl OptimizedRiskPool {
    /// Convert i128 to compact i64 representation with validation
    pub fn to_compact_amount(amount: i128) -> Result<i64, crate::ContractError> {
        if amount > i64::MAX as i128 || amount < i64::MIN as i128 {
            return Err(crate::ContractError::InvalidAmount);
        }
        Ok(amount as i64)
    }

    /// Convert compact i64 back to i128
    pub fn from_compact_amount(compact: i64) -> i128 {
        compact as i128
    }

    /// Get optimized pool statistics with caching
    pub fn get_pool_stats_optimized(env: &Env) -> Result<CompactPoolStats, crate::ContractError> {
        // Try cache first (instance storage - cheaper)
        if let Some(cached) = GasOptimizer::cache_get(env, POOL_STATS_KEY) {
            return Ok(cached);
        }

        // Fall back to persistent storage
        let stats: (i128, i128, i128, u64) = env.storage().persistent()
            .get(&crate::POOL_STATS)
            .ok_or(crate::ContractError::NotFound)?;

        let compact_stats = CompactPoolStats {
            total_liquidity: Self::to_compact_amount(stats.0)?,
            total_claims_paid: Self::to_compact_amount(stats.1)?,
            total_deposits: Self::to_compact_amount(stats.2)?,
            provider_count: stats.3 as u32,
            last_update_days: OptimizedStructures::timestamp_to_days(env.ledger().timestamp()),
        };

        // Cache for future access
        GasOptimizer::cache_set(env, POOL_STATS_KEY, &compact_stats)?;

        Ok(compact_stats)
    }

    /// Update pool statistics with minimal storage operations
    pub fn update_pool_stats_optimized(
        env: &Env,
        liquidity_change: i128,
        claims_paid_change: i128,
        deposits_change: i128,
        provider_change: i32, // Can be negative for provider removal
    ) -> Result<(), crate::ContractError> {
        // Get current stats
        let mut stats = Self::get_pool_stats_optimized(env)?;
        
        // Apply changes using safe arithmetic
        stats.total_liquidity = stats.total_liquidity
            .checked_add(Self::to_compact_amount(liquidity_change)?)
            .ok_or(crate::ContractError::Overflow)?;
        
        stats.total_claims_paid = stats.total_claims_paid
            .checked_add(Self::to_compact_amount(claims_paid_change)?)
            .ok_or(crate::ContractError::Overflow)?;
        
        stats.total_deposits = stats.total_deposits
            .checked_add(Self::to_compact_amount(deposits_change)?)
            .ok_or(crate::ContractError::Overflow)?;
        
        stats.provider_count = stats.provider_count
            .checked_add_signed(provider_change)
            .ok_or(crate::ContractError::Overflow)? as u32;
        
        stats.last_update_days = OptimizedStructures::timestamp_to_days(env.ledger().timestamp());

        // Update both cache and persistent storage
        GasOptimizer::cache_set(env, POOL_STATS_KEY, &stats)?;
        env.storage().persistent().set(&crate::POOL_STATS, &(
            Self::from_compact_amount(stats.total_liquidity),
            Self::from_compact_amount(stats.total_claims_paid),
            Self::from_compact_amount(stats.total_deposits),
            stats.provider_count as u64,
        ));

        Ok(())
    }

    /// Get compact provider info with caching
    pub fn get_provider_info_optimized(
        env: &Env,
        provider: &Address,
    ) -> Result<CompactProviderInfo, crate::ContractError> {
        // Try cache first
        let cache_key = Symbol::new(env, &format!("PROV_{}", provider.to_string().get(0..8).unwrap_or("default")));
        if let Some(cached) = GasOptimizer::cache_get(env, cache_key.clone()) {
            return Ok(cached);
        }

        // Fall back to persistent storage
        let provider_info: (i128, i128, u64) = env.storage().persistent()
            .get(&(crate::PROVIDER, provider.clone()))
            .ok_or(crate::ContractError::NotFound)?;

        let compact_info = CompactProviderInfo {
            balance: Self::to_compact_amount(provider_info.0)?,
            total_deposited: Self::to_compact_amount(provider_info.1)?,
            registered_days: OptimizedStructures::timestamp_to_days(provider_info.2),
            flags: 0u8, // Reserved for future use
        };

        // Cache for fast access
        GasOptimizer::cache_set(env, cache_key, &compact_info)?;

        Ok(compact_info)
    }

    /// Update provider information efficiently
    pub fn update_provider_info_optimized(
        env: &Env,
        provider: &Address,
        balance_change: i128,
        deposit_change: i128,
    ) -> Result<(), crate::ContractError> {
        // Get current provider info
        let mut provider_info = Self::get_provider_info_optimized(env, provider)?;
        
        // Apply changes
        provider_info.balance = provider_info.balance
            .checked_add(Self::to_compact_amount(balance_change)?)
            .ok_or(crate::ContractError::Overflow)?;
        
        provider_info.total_deposited = provider_info.total_deposited
            .checked_add(Self::to_compact_amount(deposit_change)?)
            .ok_or(crate::ContractError::Overflow)?;

        // Update cache
        let cache_key = Symbol::new(env, &format!("PROV_{}", provider.to_string().get(0..8).unwrap_or("default")));
        GasOptimizer::cache_set(env, cache_key, &provider_info)?;

        // Update persistent storage
        env.storage().persistent().set(&(crate::PROVIDER, provider.clone()), &(
            Self::from_compact_amount(provider_info.balance),
            Self::from_compact_amount(provider_info.total_deposited),
            OptimizedStructures::days_to_timestamp(provider_info.registered_days),
        ));

        Ok(())
    }

    /// Efficient batch deposit processing
    pub fn batch_deposit_liquidity_optimized(
        env: &Env,
        deposits: &Vec<(Address, i128)>,
    ) -> Result<(), crate::ContractError> {
        if deposits.is_empty() {
            return Ok(());
        }

        let mut total_liquidity_change = 0i128;
        let mut total_deposit_change = 0i128;
        let mut new_providers = 0i32;

        // Process deposits in batch
        for i in 0..deposits.len() {
            let (provider, amount) = deposits.get(i).unwrap();
            
            // Get current provider info
            let provider_exists = env.storage().persistent().has(&(crate::PROVIDER, provider.clone()));
            
            if !provider_exists {
                new_providers += 1;
            }

            // Update provider info
            Self::update_provider_info_optimized(
                env,
                &provider,
                amount,
                amount,
            )?;

            total_liquidity_change += amount;
            total_deposit_change += amount;
        }

        // Update pool statistics once
        Self::update_pool_stats_optimized(
            env,
            total_liquidity_change,
            0, // No claims paid
            total_deposit_change,
            new_providers,
        )?;

        Ok(())
    }

    /// Optimized liquidity reservation with minimal storage access
    pub fn reserve_liquidity_optimized(
        env: &Env,
        claim_id: u64,
        amount: i128,
    ) -> Result<(), crate::ContractError> {
        // Validate amount fits in compact format
        let compact_amount = Self::to_compact_amount(amount)?;

        // Get current stats using cache
        let stats = Self::get_pool_stats_optimized(env)?;
        let reserved_total: i64 = GasOptimizer::cache_get(env, RESERVED_TOTAL_KEY).unwrap_or(0i64);

        // Calculate available liquidity
        let available = stats.total_liquidity.checked_sub(reserved_total)
            .ok_or(crate::ContractError::Overflow)?;

        if available < compact_amount {
            return Err(crate::ContractError::InsufficientFunds);
        }

        // Update reserved total in cache and persistent storage
        let new_reserved = reserved_total.checked_add(compact_amount)
            .ok_or(crate::ContractError::Overflow)?;
        
        GasOptimizer::cache_set(env, RESERVED_TOTAL_KEY, &new_reserved)?;
        env.storage().persistent().set(&crate::RESERVED_TOTAL, &Self::from_compact_amount(new_reserved));

        // Store claim reservation
        env.storage().persistent().set(&(crate::CLAIM_RESERVATION, claim_id), &amount);

        Ok(())
    }

    /// Optimized claim payout with batch processing
    pub fn payout_claim_optimized(
        env: &Env,
        claim_id: u64,
        recipient: Address,
        amount: i128,
    ) -> Result<(), crate::ContractError> {
        // Validate and convert amount
        let compact_amount = Self::to_compact_amount(amount)?;

        // Get claim reservation
        let reserved_amount: i128 = env.storage().persistent()
            .get(&(crate::CLAIM_RESERVATION, claim_id))
            .ok_or(crate::ContractError::NotFound)?;

        if reserved_amount != amount {
            return Err(crate::ContractError::InvalidState);
        }

        // Update pool statistics
        Self::update_pool_stats_optimized(
            env,
            -amount, // Decrease liquidity
            amount,  // Increase claims paid
            0,       // No deposit change
            0,       // No provider change
        )?;

        // Update reserved total
        let mut reserved_total: i64 = GasOptimizer::cache_get(env, RESERVED_TOTAL_KEY).unwrap_or(0i64);
        reserved_total = reserved_total.checked_sub(compact_amount)
            .ok_or(crate::ContractError::Overflow)?;
        
        GasOptimizer::cache_set(env, RESERVED_TOTAL_KEY, &reserved_total)?;
        env.storage().persistent().set(&crate::RESERVED_TOTAL, &Self::from_compact_amount(reserved_total));

        // Remove claim reservation
        env.storage().persistent().remove(&(crate::CLAIM_RESERVATION, claim_id));

        // Emit optimized event
        env.events().publish(
            (Symbol::new(env, "claim_payout"), claim_id),
            (recipient, amount),
        );

        Ok(())
    }

    /// Efficient batch query for provider information
    pub fn get_providers_batch_optimized(
        env: &Env,
        providers: &Vec<Address>,
    ) -> Result<Vec<(Address, CompactProviderInfo)>, crate::ContractError> {
        let mut result = Vec::new(env);
        
        // Use batch operations for efficiency
        for i in 0..providers.len() {
            let provider = providers.get(i).unwrap();
            if let Ok(info) = Self::get_provider_info_optimized(env, &provider) {
                result.push_back((provider.clone(), info));
            }
        }
        
        Ok(result)
    }

    /// Clear expired cache entries to free storage
    pub fn cleanup_expired_cache(env: &Env) {
        let current_days = OptimizedStructures::timestamp_to_days(env.ledger().timestamp());
        
        // In a real implementation, we would track cache timestamps and clean up
        // entries older than CACHE_TTL_DAYS. For now, we'll just demonstrate the pattern.
        
        // This would be called periodically by a maintenance function
        // env.storage().instance().remove(&expired_cache_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_compact_amount_conversion() {
        // Test normal values
        assert_eq!(OptimizedRiskPool::to_compact_amount(1000).unwrap(), 1000i64);
        assert_eq!(OptimizedRiskPool::from_compact_amount(1000i64), 1000i128);
        
        // Test large values that fit in i64
        let large_value = i64::MAX as i128;
        assert_eq!(
            OptimizedRiskPool::to_compact_amount(large_value).unwrap() as i128,
            large_value
        );
        
        // Test values that don't fit in i64
        assert!(OptimizedRiskPool::to_compact_amount(i64::MAX as i128 + 1).is_err());
        assert!(OptimizedRiskPool::to_compact_amount(i64::MIN as i128 - 1).is_err());
    }

    #[test]
    fn test_pool_stats_optimization() {
        let env = Env::default();
        
        // Initialize with some values
        let initial_stats = (10000i128, 2000i128, 12000i128, 5u64);
        env.storage().persistent().set(&crate::POOL_STATS, &initial_stats);
        
        // Get optimized stats
        let compact_stats = OptimizedRiskPool::get_pool_stats_optimized(&env).unwrap();
        assert_eq!(compact_stats.total_liquidity, 10000i64);
        assert_eq!(compact_stats.total_claims_paid, 2000i64);
        assert_eq!(compact_stats.total_deposits, 12000i64);
        assert_eq!(compact_stats.provider_count, 5u32);
        
        // Value should be cached
        let cached: CompactPoolStats = GasOptimizer::cache_get(&env, POOL_STATS_KEY).unwrap();
        assert_eq!(cached.total_liquidity, 10000i64);
    }

    #[test]
    fn test_provider_info_optimization() {
        let env = Env::default();
        let provider = Address::generate(&env);
        
        // Initialize provider
        let provider_data = (5000i128, 5000i128, 1704067200u64);
        env.storage().persistent().set(&(crate::PROVIDER, provider.clone()), &provider_data);
        
        // Get optimized provider info
        let compact_info = OptimizedRiskPool::get_provider_info_optimized(&env, &provider).unwrap();
        assert_eq!(compact_info.balance, 5000i64);
        assert_eq!(compact_info.total_deposited, 5000i64);
        
        // Test update operation
        OptimizedRiskPool::update_provider_info_optimized(&env, &provider, 1000, 1000).unwrap();
        
        let updated_info = OptimizedRiskPool::get_provider_info_optimized(&env, &provider).unwrap();
        assert_eq!(updated_info.balance, 6000i64);
        assert_eq!(updated_info.total_deposited, 6000i64);
    }

    #[test]
    fn test_batch_operations() {
        let env = Env::default();
        let provider1 = Address::generate(&env);
        let provider2 = Address::generate(&env);
        
        // Setup initial data
        let initial_stats = (10000i128, 0i128, 10000i128, 0u64);
        env.storage().persistent().set(&crate::POOL_STATS, &initial_stats);
        
        let deposits = Vec::from_array(&env, [
            (provider1.clone(), 3000i128),
            (provider2.clone(), 2000i128),
        ]);
        
        // Process batch deposit
        OptimizedRiskPool::batch_deposit_liquidity_optimized(&env, &deposits).unwrap();
        
        // Verify results
        let stats = OptimizedRiskPool::get_pool_stats_optimized(&env).unwrap();
        assert_eq!(stats.total_liquidity, 15000i64); // 10000 + 3000 + 2000
        assert_eq!(stats.provider_count, 2u32);
    }
}