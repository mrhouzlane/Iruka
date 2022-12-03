//! Secret Orderbook Matching Engine
//!
//! Private Orderbook is a Zero-knowledge MPC prototype, where orders are matched without revealing
//! bids and asks are hidden of the users.
//!
//! This contract's flow follows as:
//!
//! 1. Initialization of contract with orderbook HashMap,
//! 2. User send their ask/bid
//! 3. Matching Engine receive the Limit for a certain Price
//! 4. Zk Computation sums yes or no for a certain Price and returns output for the User.
//! 5. When computation is complete the contract will open the output variables.
//! 6. The contract computes whether the Price/Limit is accepted or rejected.
//!
//!  

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::address::Address;
use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;
#[cfg(feature = "attestation")]
use pbc_contract_common::zk::AttestationId;
use pbc_contract_common::zk::{CalculationStatus, SecretVarId, ZkInputDef, ZkState, ZkStateChange};
use pbc_traits::ReadWriteState;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

/// The maximum size of MPC variables.
const BITLENGTH_OF_SECRET_VOTE_VARIABLES: u32 = 32;

#[derive(Debug)]
enum BidOrAsk {
    Bid,
    Ask,
}

/// Defintion of the orderbook
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, Clone)]
struct Orderbook {
    asks: HashMap<Price, Limit>,
    bids: HashMap<Price, Limit>,
}

struct Order {
    size: f64,
    bid_or_ask: BidOrAsk,
}

struct User {
    user_id: u64,
    matching_status: bool,
}

/// Defintion of a Zk-Order
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, Clone)]
struct ZKOrder {
    order: HashMap<Order, User>,
}

#[derive(ReadWriteState, CreateTypeSpec, Clone)]
struct MatchingEngine {}
