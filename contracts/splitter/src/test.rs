#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env};

struct Setup<'a> {
    env: Env,
    splitter: SplitterClient<'a>,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
    contract_id: Address,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let contract_id = env.register(Splitter, ());
    Setup {
        splitter: SplitterClient::new(&env, &contract_id),
        token: token::Client::new(&env, &sac.address()),
        token_admin: token::StellarAssetClient::new(&env, &sac.address()),
        contract_id,
        env,
    }
}

#[test]
fn splits_by_share() {
    let s = setup();
    let (a, b) = (Address::generate(&s.env), Address::generate(&s.env));
    s.splitter
        .init(&s.token.address, &vec![&s.env, a.clone(), b.clone()], &vec![&s.env, 3, 1]);
    s.token_admin.mint(&s.contract_id, &1000);

    assert_eq!(s.splitter.distribute(), 1000);
    assert_eq!(s.token.balance(&a), 750);
    assert_eq!(s.token.balance(&b), 250);
    assert_eq!(s.token.balance(&s.contract_id), 0);
}

#[test]
fn rounding_dust_goes_to_last_recipient() {
    let s = setup();
    let (a, b, c) = (
        Address::generate(&s.env),
        Address::generate(&s.env),
        Address::generate(&s.env),
    );
    s.splitter.init(
        &s.token.address,
        &vec![&s.env, a.clone(), b.clone(), c.clone()],
        &vec![&s.env, 1, 1, 1],
    );
    s.token_admin.mint(&s.contract_id, &100);
    s.splitter.distribute();

    assert_eq!(s.token.balance(&a), 33);
    assert_eq!(s.token.balance(&b), 33);
    assert_eq!(s.token.balance(&c), 34);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn cannot_init_twice() {
    let s = setup();
    let a = Address::generate(&s.env);
    let r = vec![&s.env, a];
    let sh = vec![&s.env, 1u32];
    s.splitter.init(&s.token.address, &r, &sh);
    s.splitter.init(&s.token.address, &r, &sh);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn rejects_zero_share() {
    let s = setup();
    let a = Address::generate(&s.env);
    s.splitter
        .init(&s.token.address, &vec![&s.env, a], &vec![&s.env, 0u32]);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn empty_balance_is_rejected() {
    let s = setup();
    let a = Address::generate(&s.env);
    s.splitter
        .init(&s.token.address, &vec![&s.env, a], &vec![&s.env, 1u32]);
    s.splitter.distribute();
}
