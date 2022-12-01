//! This is the example voting contract.
//! For more information on how to build it see README.md
#![allow(unused_variables)]

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;

use std::collections::{BTreeMap, BTreeSet};

use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;

/// This is the state of the contract which is persisted on chain.
///
/// The #\[state\] macro generates serialization logic for the struct.
///
/// # Members
///
/// * `proposal_id`: [`u64`] -  the identification of the proposal.
/// * `mp_addresses`: [`Vec`]<[`Address`]> - the list of legal voters.
/// * `votes`: [`BTreeMap`]<[`Address`], [`u8`]> - the votes that have already been cast.
/// * `closed`: [`u8`] - bool to determine if the poll is over.
///
#[state]
pub struct VotingContractState {
    proposal_id: u64,
    mp_addresses: Vec<Address>,
    votes: BTreeMap<Address, u8>,
    closed: u8,
}

impl VotingContractState {
    fn register_vote(&mut self, address: Address, vote: u8) {
        self.votes.insert(address, vote);
    }

    fn close_if_finished(&mut self) {
        if self.votes.len() == self.mp_addresses.len() {
            self.closed = 1;
        };
    }
}

/// This is the main action of the contract in which the sender can cast a vote.
///
///
/// # Parameters
///
/// * `ctx`: [`ContractContext`] - the contract context containing sender and chain information.
/// * `proposal_id`: [`u64`] - the id of the proposal.
/// * `mp_addresses`: [`u64`] - the list of legal voters.
///
/// # Returns
///
/// The return value is the new state and an empty list of events.
///
#[action]
pub fn vote(
    context: ContractContext,
    state: VotingContractState,
    vote: u8,
) -> (VotingContractState, Vec<EventGroup>) {
    assert_eq!(state.closed, 0, "The poll is closed");
    assert!(
        state.mp_addresses.contains(&context.sender),
        "Only members of the parliament can vote"
    );
    assert!(
        vote == 0 || vote == 1,
        "Only \"yes\" and \"no\" votes are allowed"
    );

    let mut new_state = state;
    new_state.register_vote(context.sender, vote);
    new_state.close_if_finished();
    (new_state, vec![])
}

/// Initial function to bootstrap the contract's state. Must return a the (state-struct, events).
///
/// # Parameters
///
/// * `ctx`: [`ContractContext`] - the contract context containing sender and chain information.
/// * `proposal_id`: [`u64`] - the id of the proposal.
/// * `mp_addresses`: [`u64`] - the list of legal voters.
///
/// # Returns
///
/// The new state object of type [`TokenContractState`] with an updated `state.votes` and an empty
/// list of events.
///
#[init]
pub fn initialize(
    _ctx: ContractContext,
    proposal_id: u64,
    mp_addresses: Vec<Address>,
) -> (VotingContractState, Vec<EventGroup>) {
    assert_ne!(
        mp_addresses.len(),
        0,
        "Cannot start a poll without parliament members"
    );

    let mut address_set = BTreeSet::new();
    for mp_address in mp_addresses.iter() {
        address_set.insert(*mp_address);
    }
    assert_eq!(
        mp_addresses.len(),
        address_set.len(),
        "Duplicate MP address in input"
    );

    let state = VotingContractState {
        proposal_id,
        mp_addresses,
        votes: BTreeMap::new(),
        closed: 0,
    };
    (state, vec![])
}
