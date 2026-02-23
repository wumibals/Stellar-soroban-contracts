use soroban_sdk::{symbol_short, token, Address, Env};

use crate::errors::Error;
use crate::ins_token;
use crate::storage;
use crate::types::Listing;

pub fn list_for_sale(
    env: &Env,
    seller: &Address,
    policy_id: u64,
    amount: i128,
    price_per_token: i128,
) -> Result<u64, Error> {
    if amount <= 0 || price_per_token <= 0 {
        return Err(Error::InvalidAmount);
    }

    let balance = storage::get_balance(env, seller, policy_id);
    if balance < amount {
        return Err(Error::InsufficientBalance);
    }

    ins_token::transfer(env, seller, &env.current_contract_address(), policy_id, amount)?;

    let count = storage::get_listing_count(env);
    let listing_id = count + 1;

    let listing = Listing {
        id: listing_id,
        seller: seller.clone(),
        policy_id,
        amount,
        price_per_token,
        is_active: true,
    };

    storage::set_listing(env, &listing);
    storage::set_listing_count(env, listing_id);

    env.events().publish(
        (symbol_short!("listed"), seller.clone()),
        (listing_id, policy_id, amount, price_per_token),
    );

    Ok(listing_id)
}

pub fn buy_listing(
    env: &Env,
    buyer: &Address,
    listing_id: u64,
    amount: i128,
    base_token: &Address,
) -> Result<(), Error> {
    let mut listing = storage::get_listing(env, listing_id)?;

    if !listing.is_active {
        return Err(Error::ListingNotActive);
    }
    if amount <= 0 || amount > listing.amount {
        return Err(Error::InvalidAmount);
    }

    let total_cost = amount
        .checked_mul(listing.price_per_token)
        .ok_or(Error::Overflow)?;

    let base_client = token::Client::new(env, base_token);
    base_client.transfer(buyer, &listing.seller, &total_cost);

    ins_token::transfer(
        env,
        &env.current_contract_address(),
        buyer,
        listing.policy_id,
        amount,
    )?;

    listing.amount -= amount;
    if listing.amount == 0 {
        listing.is_active = false;
    }
    storage::set_listing(env, &listing);

    env.events().publish(
        (symbol_short!("sold"), buyer.clone()),
        (listing_id, amount, total_cost),
    );

    Ok(())
}

pub fn cancel_listing(env: &Env, seller: &Address, listing_id: u64) -> Result<(), Error> {
    let mut listing = storage::get_listing(env, listing_id)?;

    if listing.seller != *seller {
        return Err(Error::Unauthorized);
    }
    if !listing.is_active {
        return Err(Error::ListingNotActive);
    }

    ins_token::transfer(
        env,
        &env.current_contract_address(),
        seller,
        listing.policy_id,
        listing.amount,
    )?;

    listing.is_active = false;
    storage::set_listing(env, &listing);

    env.events().publish(
        (symbol_short!("cancelled"), seller.clone()),
        listing_id,
    );

    Ok(())
}
