use soroban_sdk::{symbol_short, token, Address, Env};

use crate::errors::Error;
use crate::ins_token;
use crate::storage;

const MINIMUM_LIQUIDITY: i128 = 1_000;
const FEE_PRECISION: i128 = 10_000;

pub fn add_liquidity(
    env: &Env,
    provider: &Address,
    policy_id: u64,
    insurance_amount: i128,
    base_amount: i128,
    base_token: &Address,
) -> Result<i128, Error> {
    if insurance_amount <= 0 || base_amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let mut pool = storage::get_pool(env);

    let lp_tokens = if pool.total_lp_tokens == 0 {
        let lp = isqrt(
            insurance_amount
                .checked_mul(base_amount)
                .ok_or(Error::Overflow)?,
        );
        if lp <= MINIMUM_LIQUIDITY {
            return Err(Error::InsufficientLiquidity);
        }
        let locked = MINIMUM_LIQUIDITY;
        let locked_lp = storage::get_lp_balance(env, &env.current_contract_address());
        storage::set_lp_balance(env, &env.current_contract_address(), locked_lp + locked);
        lp - locked
    } else {
        let lp_from_ins = insurance_amount
            .checked_mul(pool.total_lp_tokens)
            .ok_or(Error::Overflow)?
            / pool.insurance_reserve;
        let lp_from_base = base_amount
            .checked_mul(pool.total_lp_tokens)
            .ok_or(Error::Overflow)?
            / pool.base_reserve;
        lp_from_ins.min(lp_from_base)
    };

    if lp_tokens <= 0 {
        return Err(Error::InsufficientLiquidity);
    }

    ins_token::transfer(
        env,
        provider,
        &env.current_contract_address(),
        policy_id,
        insurance_amount,
    )?;

    let base_client = token::Client::new(env, base_token);
    base_client.transfer(provider, &env.current_contract_address(), &base_amount);

    pool.insurance_reserve += insurance_amount;
    pool.base_reserve += base_amount;
    pool.total_lp_tokens += lp_tokens;
    storage::set_pool(env, &pool);

    let current_lp = storage::get_lp_balance(env, provider);
    storage::set_lp_balance(env, provider, current_lp + lp_tokens);

    env.events().publish(
        (symbol_short!("add_liq"), provider.clone()),
        (insurance_amount, base_amount, lp_tokens),
    );

    Ok(lp_tokens)
}

pub fn remove_liquidity(
    env: &Env,
    provider: &Address,
    policy_id: u64,
    lp_amount: i128,
    base_token: &Address,
) -> Result<(i128, i128), Error> {
    if lp_amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let lp_balance = storage::get_lp_balance(env, provider);
    if lp_balance < lp_amount {
        return Err(Error::InsufficientBalance);
    }

    let mut pool = storage::get_pool(env);

    let insurance_out = lp_amount
        .checked_mul(pool.insurance_reserve)
        .ok_or(Error::Overflow)?
        / pool.total_lp_tokens;

    let base_out = lp_amount
        .checked_mul(pool.base_reserve)
        .ok_or(Error::Overflow)?
        / pool.total_lp_tokens;

    if insurance_out == 0 || base_out == 0 {
        return Err(Error::InsufficientLiquidity);
    }

    pool.insurance_reserve -= insurance_out;
    pool.base_reserve -= base_out;
    pool.total_lp_tokens -= lp_amount;
    storage::set_pool(env, &pool);

    storage::set_lp_balance(env, provider, lp_balance - lp_amount);

    ins_token::transfer(
        env,
        &env.current_contract_address(),
        provider,
        policy_id,
        insurance_out,
    )?;

    let base_client = token::Client::new(env, base_token);
    base_client.transfer(&env.current_contract_address(), provider, &base_out);

    env.events().publish(
        (symbol_short!("rem_liq"), provider.clone()),
        (insurance_out, base_out, lp_amount),
    );

    Ok((insurance_out, base_out))
}

pub fn swap(
    env: &Env,
    user: &Address,
    policy_id: u64,
    amount_in: i128,
    min_amount_out: i128,
    insurance_to_base: bool,
    base_token: &Address,
) -> Result<i128, Error> {
    if amount_in <= 0 {
        return Err(Error::InvalidAmount);
    }

    let mut pool = storage::get_pool(env);

    if pool.insurance_reserve == 0 || pool.base_reserve == 0 {
        return Err(Error::InsufficientLiquidity);
    }

    let fee = amount_in
        .checked_mul(pool.fee_rate as i128)
        .ok_or(Error::Overflow)?
        / FEE_PRECISION;
    let amount_in_after_fee = amount_in - fee;

    let amount_out = if insurance_to_base {
        get_amount_out(amount_in_after_fee, pool.insurance_reserve, pool.base_reserve)?
    } else {
        get_amount_out(amount_in_after_fee, pool.base_reserve, pool.insurance_reserve)?
    };

    if amount_out < min_amount_out {
        return Err(Error::SlippageExceeded);
    }

    if insurance_to_base {
        ins_token::transfer(
            env,
            user,
            &env.current_contract_address(),
            policy_id,
            amount_in,
        )?;
        pool.insurance_reserve += amount_in;
        pool.base_reserve -= amount_out;

        let base_client = token::Client::new(env, base_token);
        base_client.transfer(&env.current_contract_address(), user, &amount_out);
    } else {
        let base_client = token::Client::new(env, base_token);
        base_client.transfer(user, &env.current_contract_address(), &amount_in);
        pool.base_reserve += amount_in;
        pool.insurance_reserve -= amount_out;

        ins_token::transfer(
            env,
            &env.current_contract_address(),
            user,
            policy_id,
            amount_out,
        )?;
    }

    storage::set_pool(env, &pool);

    env.events().publish(
        (symbol_short!("swap"), user.clone()),
        (amount_in, amount_out, insurance_to_base),
    );

    Ok(amount_out)
}

pub fn get_quote(
    env: &Env,
    amount_in: i128,
    insurance_to_base: bool,
) -> Result<i128, Error> {
    if amount_in <= 0 {
        return Err(Error::InvalidAmount);
    }

    let pool = storage::get_pool(env);

    let fee = amount_in
        .checked_mul(pool.fee_rate as i128)
        .ok_or(Error::Overflow)?
        / FEE_PRECISION;
    let amount_in_after_fee = amount_in - fee;

    if insurance_to_base {
        get_amount_out(amount_in_after_fee, pool.insurance_reserve, pool.base_reserve)
    } else {
        get_amount_out(amount_in_after_fee, pool.base_reserve, pool.insurance_reserve)
    }
}

fn get_amount_out(
    amount_in: i128,
    reserve_in: i128,
    reserve_out: i128,
) -> Result<i128, Error> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(Error::InsufficientLiquidity);
    }
    let numerator = amount_in
        .checked_mul(reserve_out)
        .ok_or(Error::Overflow)?;
    let denominator = reserve_in
        .checked_add(amount_in)
        .ok_or(Error::Overflow)?;
    if denominator == 0 {
        return Err(Error::DivisionByZero);
    }
    Ok(numerator / denominator)
}

fn isqrt(x: i128) -> i128 {
    if x == 0 {
        return 0;
    }
    let mut z = (x + 1) / 2;
    let mut y = x;
    while z < y {
        y = z;
        z = (x / z + z) / 2;
    }
    y
}
