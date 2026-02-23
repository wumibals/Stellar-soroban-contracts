use soroban_sdk::{symbol_short, Address, Env};

use crate::errors::Error;
use crate::storage;

pub fn mint(env: &Env, to: &Address, policy_id: u64, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let balance = storage::get_balance(env, to, policy_id);
    storage::set_balance(env, to, policy_id, balance + amount);

    let supply = storage::get_total_supply(env, policy_id);
    storage::set_total_supply(env, policy_id, supply + amount);

    env.events()
        .publish((symbol_short!("mint"), to.clone()), (policy_id, amount));

    Ok(())
}

pub fn burn(env: &Env, from: &Address, policy_id: u64, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let balance = storage::get_balance(env, from, policy_id);
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }

    storage::set_balance(env, from, policy_id, balance - amount);

    let supply = storage::get_total_supply(env, policy_id);
    storage::set_total_supply(env, policy_id, supply - amount);

    env.events()
        .publish((symbol_short!("burn"), from.clone()), (policy_id, amount));

    Ok(())
}

pub fn transfer(
    env: &Env,
    from: &Address,
    to: &Address,
    policy_id: u64,
    amount: i128,
) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let from_balance = storage::get_balance(env, from, policy_id);
    if from_balance < amount {
        return Err(Error::InsufficientBalance);
    }

    storage::set_balance(env, from, policy_id, from_balance - amount);

    let to_balance = storage::get_balance(env, to, policy_id);
    storage::set_balance(env, to, policy_id, to_balance + amount);

    env.events().publish(
        (symbol_short!("transfer"), from.clone()),
        (to.clone(), policy_id, amount),
    );

    Ok(())
}

pub fn approve(
    env: &Env,
    owner: &Address,
    spender: &Address,
    policy_id: u64,
    amount: i128,
) -> Result<(), Error> {
    if amount < 0 {
        return Err(Error::InvalidAmount);
    }

    storage::set_allowance(env, owner, spender, policy_id, amount);

    env.events().publish(
        (symbol_short!("approve"), owner.clone()),
        (spender.clone(), policy_id, amount),
    );

    Ok(())
}

pub fn transfer_from(
    env: &Env,
    spender: &Address,
    from: &Address,
    to: &Address,
    policy_id: u64,
    amount: i128,
) -> Result<(), Error> {
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let allowance = storage::get_allowance(env, from, spender, policy_id);
    if allowance < amount {
        return Err(Error::InsufficientBalance);
    }

    storage::set_allowance(env, from, spender, policy_id, allowance - amount);
    transfer(env, from, to, policy_id, amount)
}
