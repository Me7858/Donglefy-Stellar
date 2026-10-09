#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env,
    Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidShares = 3,
    NothingToDistribute = 4,
}

#[contracttype]
enum DataKey {
    Token,
    Recipients,
    Shares,
    TotalShares,
}

#[contract]
pub struct Splitter;

#[contractimpl]
impl Splitter {
    /// Configure the splitter once. `shares[i]` is the weight of `recipients[i]`.
    pub fn init(env: Env, token: Address, recipients: Vec<Address>, shares: Vec<u32>) {
        if env.storage().instance().has(&DataKey::Token) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        if recipients.is_empty() || recipients.len() != shares.len() {
            panic_with_error!(&env, Error::InvalidShares);
        }
        let mut total: u32 = 0;
        for s in shares.iter() {
            if s == 0 {
                panic_with_error!(&env, Error::InvalidShares);
            }
            total = total
                .checked_add(s)
                .unwrap_or_else(|| panic_with_error!(&env, Error::InvalidShares));
        }
        let storage = env.storage().instance();
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::Recipients, &recipients);
        storage.set(&DataKey::Shares, &shares);
        storage.set(&DataKey::TotalShares, &total);
    }

    /// Pay out the contract's entire token balance by share. Anyone may call it.
    /// Rounding dust goes to the last recipient so the balance always reaches zero.
    pub fn distribute(env: Env) -> i128 {
        let storage = env.storage().instance();
        let token_addr: Address = storage
            .get(&DataKey::Token)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));
        let recipients: Vec<Address> = storage.get(&DataKey::Recipients).unwrap();
        let shares: Vec<u32> = storage.get(&DataKey::Shares).unwrap();
        let total: u32 = storage.get(&DataKey::TotalShares).unwrap();

        let this = env.current_contract_address();
        let client = token::Client::new(&env, &token_addr);
        let balance = client.balance(&this);
        if balance <= 0 {
            panic_with_error!(&env, Error::NothingToDistribute);
        }

        let last = recipients.len() - 1;
        let mut paid: i128 = 0;
        for i in 0..recipients.len() {
            let amount = if i == last {
                balance - paid
            } else {
                balance
                    .checked_mul(shares.get(i).unwrap() as i128)
                    .unwrap_or_else(|| panic_with_error!(&env, Error::InvalidShares))
                    / (total as i128)
            };
            if amount > 0 {
                client.transfer(&this, &recipients.get(i).unwrap(), &amount);
            }
            paid += amount;
        }
        balance
    }

    pub fn recipients(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Recipients)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }

    pub fn shares(env: Env) -> Vec<u32> {
        env.storage()
            .instance()
            .get(&DataKey::Shares)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }
}

#[cfg(test)]
mod test;
