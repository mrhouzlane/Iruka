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

/// This contract's state
#[state]
struct ContractState {}
