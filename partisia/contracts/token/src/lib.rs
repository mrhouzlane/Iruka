//! This is an example token smart contract.
//!
//! The contract has a constant total supply of tokens.
//! The total supply is initialized together with the contract.
//!
//! Any token owner can then `transfer` tokens to other accounts, or `approve` other accounts to use their tokens.
//! If a Alice has been approved tokens from Bob, then Alice can use `transfer_from` to use Bob's tokens.
//!
//! The contract is inspired by the ERC20 token contract.\
//! <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-20.md>
#![allow(unused_variables)]

#[macro_use]
extern crate pbc_contract_codegen;

use create_type_spec_derive::CreateTypeSpec;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;
use std::collections::BTreeMap;
use std::ops::Add;

use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;



/// Custom struct for the state of the contract.
///
/// The "state" attribute is attached.
///
/// ### Fields:
///
/// * `name`: [`String`], the name of the token - e.g. "MyToken".\
///
/// * `symbol`: [`String`], the symbol of the token. E.g. "HIX".\
///
/// * `decimals`: [`u8`], the number of decimals the token uses - e.g. 8,
/// means to divide the token amount by `100000000` to get its user representation.\
///
/// * `owner`: [`Address`], owner of the contract.
///
/// * `total_supply`: [`u64`], current amount of tokens for the TokenContract.
///
/// * `balances`: [`BTreeMap<Address, u64>`], ledger for the accounts associated with the contract.
///
/// * `allowed`: [`BTreeMap<Address, BTreeMap<Address, u64>>`], allowance from an owner to a spender.
#[state]
pub struct TokenContractState {
    name: String,
    decimals: u8,
    symbol: String,
    owner: Address,
    total_supply: u64,
    balances: BTreeMap<Address, u64>,
    allowed: BTreeMap<Address, BTreeMap<Address, u64>>,
}

impl TokenContractState {
    /// Gets the balance of the specified address.
    ///
    /// ### Parameters:
    ///
    /// * `owner`: The [`Address`] to query the balance of.
    ///
    /// ### Returns:
    ///
    /// An [`u64`] representing the amount owned by the passed address.
    pub fn balance_of(&mut self, owner: Address) -> u64 {
        *self.balances.entry(owner).or_insert(0)
    }

    /// Function to check the amount of tokens that an owner allowed to a spender.
    ///
    /// ### Parameters:
    ///
    /// * `owner`: [`Address`] The address which owns the funds.
    ///
    /// * `spender`: [`Address`] The address which will spend the funds.
    ///
    /// ### Returns:
    ///
    /// A [`u64`] specifying the amount whicher `spender` is still allowed to withdraw from `owner`.
    pub fn allowance(&mut self, owner: Address, spender: Address) -> u64 {
        let allowed_from_owner = self.allowed.entry(owner).or_insert_with(BTreeMap::new);
        let allowance = allowed_from_owner.entry(spender).or_insert(0);
        *allowance
    }

    fn update_allowance(&mut self, owner: Address, spender: Address, value: u64) {
        let allowed_from_owner = self.allowed.entry(owner).or_insert_with(BTreeMap::new);
        allowed_from_owner.insert(spender, value);
    }
}

/// Initial function to bootstrap the contracts state. Must return the state-struct.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], initial context.
///
/// * `name`: [`String`], the name of the token - e.g. "MyToken".\
///
/// * `symbol`: [`String`], the symbol of the token. E.g. "HIX".\
///
/// * `decimals`: [`u8`], the number of decimals the token uses - e.g. 8,
/// means to divide the token amount by `100000000` to get its user representation.\
///
/// * `total_supply`: [`u64`], current amount of tokens for the TokenContract.
///
/// ### Returns:
///
/// The new state object of type [`TokenContractState`] with an initialized ledger.
#[init]
pub fn initialize(
    ctx: ContractContext,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut balances = BTreeMap::new();
    balances.insert(ctx.sender, total_supply);

    let state = TokenContractState {
        name,
        symbol,
        decimals,
        owner: ctx.sender,
        total_supply,
        balances,
        allowed: BTreeMap::new(),
    };

    (state, vec![])
}

/// Represents the type of a transfer.
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, Clone)]
pub struct Transfer {
    /// The address to transfer to.
    pub to: Address,
    /// The amount to transfer.
    pub value: u64,
}

/// Transfers `value` amount of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `value`: [`u64`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
#[action(shortname = 0x01)]
pub fn transfer(
    context: ContractContext,
    state: TokenContractState,
    to: Address,
    value: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    core_transfer(context.sender, state, to, value)
}

/// Transfers a bulk of `value` amount of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `transfers`: [`Vec[Transfer]`], vector of [the address to transfer to, amount to transfer].
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
#[action(shortname = 0x02)]
pub fn bulk_transfer(
    context: ContractContext,
    state: TokenContractState,
    transfers: Vec<Transfer>,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut new_state = state;
    for t in transfers {
        new_state = core_transfer(context.sender, new_state, t.to, t.value).0;
    }
    (new_state, vec![])
}

/// Transfers `value` amount of tokens from address `from` to address `to`.\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `value`: [`u64`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
#[action(shortname = 0x03)]
pub fn transfer_from(
    context: ContractContext,
    state: TokenContractState,
    from: Address,
    to: Address,
    value: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    core_transfer_from(context.sender, state, from, to, value)
}

/// Transfers a bulk of `value` amount of tokens to address `to` from address `from` .\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `transfers`: [`Vec[Transfer]`], vector of [the address to transfer to, amount to transfer].
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
#[action(shortname = 0x04)]
pub fn bulk_transfer_from(
    context: ContractContext,
    state: TokenContractState,
    from: Address,
    transfers: Vec<Transfer>,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut new_state = state;
    for t in transfers {
        new_state = core_transfer_from(context.sender, new_state, from, t.to, t.value).0;
    }
    (new_state, vec![])
}

/// Allows `spender` to withdraw from the owners account multiple times, up to the `value` amount.
/// If this function is called again it overwrites the current allowance with `value`.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `spender`: [`Address`], the address of the spender.
///
/// * `value`: [`u64`], approved amount.
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
#[action(shortname = 0x05)]
pub fn approve(
    context: ContractContext,
    state: TokenContractState,
    spender: Address,
    value: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut new_state = state;
    new_state.update_allowance(context.sender, spender, value);
    (new_state, vec![])
}

/// Transfers `value` amount of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `sender`: [`Address`], the sender of the transaction.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `value`: [`u64`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
pub fn core_transfer(
    sender: Address,
    state: TokenContractState,
    to: Address,
    value: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut new_state = state;
    let from_amount = new_state.balance_of(sender);
    let o_new_from_amount = from_amount.checked_sub(value);
    match o_new_from_amount {
        Some(new_from_amount) => {
            new_state.balances.insert(sender, new_from_amount);
        }
        None => {
            panic!("Underflow in transfer - owner did not have enough tokens");
        }
    }
    let to_amount = new_state.balance_of(to);
    new_state.balances.insert(to, to_amount.add(value));
    if new_state.balance_of(sender) == 0 {
        new_state.balances.remove(&sender);
    };
    (new_state, vec![])
}

/// Transfers `value` amount of tokens from address `from` to address `to`.\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `sender`: [`Address`], the sender of the transaction.
///
/// * `state`: [`TokenContractState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `value`: [`u64`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenContractState`] with an updated ledger.
pub fn core_transfer_from(
    sender: Address,
    state: TokenContractState,
    from: Address,
    to: Address,
    value: u64,
) -> (TokenContractState, Vec<EventGroup>) {
    let mut new_state = state;
    let from_allowed = new_state.allowance(from, sender);
    let o_new_allowed_amount = from_allowed.checked_sub(value);
    match o_new_allowed_amount {
        Some(new_allowed_amount) => {
            new_state.update_allowance(from, sender, new_allowed_amount);
        }
        None => {
            panic!("Underflow in transfer_from - tokens has not been approved for transfer");
        }
    }
    core_transfer(from, new_state, to, value)
}
