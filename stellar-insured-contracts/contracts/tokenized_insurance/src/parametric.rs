use soroban_sdk::{symbol_short, Env};

use crate::errors::Error;
use crate::storage;
use crate::types::{ParametricTrigger, PolicyStatus, PolicyType, TriggerType};

pub fn register_trigger(
    env: &Env,
    trigger_type: TriggerType,
    policy_type: PolicyType,
    threshold: i128,
    is_above_threshold: bool,
) -> Result<u64, Error> {
    let count = storage::get_trigger_count(env);
    let trigger_id = count + 1;

    let trigger = ParametricTrigger {
        id: trigger_id,
        trigger_type,
        threshold,
        policy_type,
        is_triggered: false,
        last_value: 0,
        is_above_threshold,
    };

    storage::set_trigger(env, &trigger);
    storage::set_trigger_count(env, trigger_id);

    env.events()
        .publish((symbol_short!("trg_reg"),), (trigger_id, threshold));

    Ok(trigger_id)
}

pub fn submit_oracle_data(env: &Env, trigger_id: u64, value: i128) -> Result<bool, Error> {
    let mut trigger = storage::get_trigger(env, trigger_id)?;

    trigger.last_value = value;

    let condition_met = if trigger.is_above_threshold {
        value > trigger.threshold
    } else {
        value < trigger.threshold
    };

    if condition_met && !trigger.is_triggered {
        trigger.is_triggered = true;
        storage::set_trigger(env, &trigger);

        env.events().publish(
            (symbol_short!("trg_fire"),),
            (trigger_id, value, trigger.threshold),
        );

        return Ok(true);
    }

    storage::set_trigger(env, &trigger);
    Ok(false)
}

pub fn check_and_payout(env: &Env, policy_id: u64) -> Result<bool, Error> {
    let policy = storage::get_policy(env, policy_id)?;

    if policy.status != PolicyStatus::Active {
        return Ok(false);
    }

    if policy.trigger_id == 0 {
        return Ok(false);
    }

    let trigger = storage::get_trigger(env, policy.trigger_id)?;

    if trigger.is_triggered && trigger.policy_type == policy.policy_type {
        let mut updated = policy.clone();
        updated.status = PolicyStatus::Claimed;
        storage::set_policy(env, &updated);

        env.events().publish(
            (symbol_short!("par_pay"),),
            (
                policy_id,
                updated.coverage_amount,
                updated.policyholder.clone(),
            ),
        );

        return Ok(true);
    }

    Ok(false)
}
