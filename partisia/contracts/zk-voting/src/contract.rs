//! Secret Voting Contract
//!
//! Secret voting is a common Zero-knowledge MPC example, wherein several persons are interested in
//! voting upon some question, without revealing their personal preference, similar to many
//! democratic election processes.
//!
//! This contract's flow follows as:
//!
//! 1. Initialization of contract with voting information, including voting treshhold,
//!    administrator, voting addresses, and minimum voting period.
//! 2. Voters send their votes. (0 is against, any other value is for)
//! 3. At some point after the minimum voting period, the administrator starts the voting counting
//!    process.
//! 4. Zk Computation sums yes votes and no votes, and output each as a separate variable.
//! 5. When computation is complete the contract will open the output variables.
//! 6. The contract computes whether the vote was accepted or rejected.

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;
#[cfg(feature = "attestation")]
use pbc_contract_common::zk::AttestationId;
use pbc_contract_common::zk::{CalculationStatus, SecretVarId, ZkInputDef, ZkState, ZkStateChange};
use pbc_traits::ReadWriteState;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

mod fraction;

use fraction::Fraction;

/// Secret variable metadata. Unused for this contract, so we use a zero-sized struct to save space.
#[derive(ReadWriteState, Debug)]
#[repr(C)]
struct SecretVarMetadata {
    variable_type: SecretVarType,
}

#[derive(ReadWriteState, Debug, PartialEq)]
#[repr(u8)]
enum SecretVarType {
    Vote = 1,
    CountedYesVotes = 2,
}

/// The maximum size of MPC variables.
const BITLENGTH_OF_SECRET_VOTE_VARIABLES: u32 = 32;

/// Definition of the voting rules
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, Clone)]
struct VoteBasis {
    /// Fraction, strictly more required
    required_ratio: Fraction,
    /// Whether to count non-voting voters in the sum of votes given.
    absent_as_against: bool,
}

impl VoteBasis {
    const _EXAMPLE_MAJORITY: VoteBasis = VoteBasis {
        required_ratio: unsafe { Fraction::new_unchecked(1, 2) },
        absent_as_against: false,
    };
    const _EXAMPLE_STRICT_MAJORITY: VoteBasis = VoteBasis {
        required_ratio: unsafe { Fraction::new_unchecked(1, 2) },
        absent_as_against: true,
    };
    const _EXAMPLE_STRICT_SUPERMAJORITY: VoteBasis = VoteBasis {
        required_ratio: unsafe { Fraction::new_unchecked(2, 3) },
        absent_as_against: true,
    };
}

#[derive(ReadWriteState, CreateTypeSpec, Clone)]
struct VoteResult {
    votes_for: u32,
    votes_against: u32,
    passed: bool,
}

impl VoteBasis {
    fn assert_valid(&self) {
        self.required_ratio.assert_valid()
    }
}

/// This contract's state
#[state]
struct ContractState {
    /// Address allowed to start computation
    administrator: Address,
    /// When the voting stops; at this point all inputs must have been made, though not necessarily
    /// finalized.
    ///
    /// Represented as milliseconds since the epoche.
    deadline_voting_time: i64,
    /// When the vote counting is allowed to start; the administrator cannot start the counting
    /// before this point in time. The discrepency between [`deadline_voting_time`] and
    /// [`deadline_commitment_time`] is to allow inputs declared before [`deadline_voting_time`] to
    /// be commited, as [`deadline_commitment_time`] will throw out pending inputs.
    ///
    /// Represented as milliseconds since the epoche.
    deadline_commitment_time: i64,
    /// Allowed voting addresses
    allowed_voters: Vec<Address>,

    /// Definition of the voting rules
    vote_definition: VoteBasis,

    vote_result: Option<VoteResult>,
}

/// Number of milliseconds between closing for inputs, and when the counting can start at the
/// earliest.
///
/// Milliseconds equal to an hour.
const ESTIMATED_MAX_INPUT_COMMITMENT_DURATION_MS: i64 = 60 * 60 * 1000;

/// Initializes contract
///
/// Note that administrator is set to whoever initializes the contact.
#[init]
fn initialize(
    ctx: ContractContext,
    _zk_state: ZkState<SecretVarMetadata>,
    voting_duration_ms: u32,
    allowed_voters: Vec<Address>,
    vote_definition: VoteBasis,
) -> ContractState {
    vote_definition.assert_valid();
    let deadline_voting_time = ctx.block_production_time + (voting_duration_ms as i64);
    let deadline_commitment_time =
        deadline_voting_time + ESTIMATED_MAX_INPUT_COMMITMENT_DURATION_MS;
    ContractState {
        administrator: ctx.sender,
        deadline_voting_time,
        deadline_commitment_time,
        allowed_voters,
        vote_definition,
        vote_result: None,
    }
}

/// Adds another vote.
///
/// The ZkInputDef encodes that the variable should have size [`BITLENGTH_OF_SECRET_VOTE_VARIABLES`].
#[zk_on_secret_input(shortname = 0x40)]
fn add_vote(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (
    ContractState,
    Vec<EventGroup>,
    ZkInputDef<SecretVarMetadata>,
) {
    assert!(
        context.block_production_time < state.deadline_voting_time,
        "Not allowed to vote after the deadline at {} ms UTC, current time is {} ms UTC",
        state.deadline_commitment_time,
        context.block_production_time,
    );
    assert!(
        state.allowed_voters.contains(&context.sender),
        "Only voters can send votes.",
    );
    assert!(
        zk_state
            .secret_variables
            .iter()
            .chain(zk_state.pending_inputs.iter())
            .all(|v| v.owner != context.sender),
        "Each voter is only allowed to send one vote variable. Sender: {:?}",
        context.sender
    );
    let input_def = ZkInputDef {
        seal: false,
        metadata: SecretVarMetadata {
            variable_type: SecretVarType::Vote,
        },
        expected_bit_lengths: vec![BITLENGTH_OF_SECRET_VOTE_VARIABLES],
    };
    (state, vec![], input_def)
}

/// Allows anybody to start the computation of the vote, but only after the counting period.
///
/// The vote computation is automatic beyond this call, involving several steps, as described in the module documentation.
///
/// NOTE: This will remove all pending inputs
#[action(shortname = 0x01)]
fn start_vote_counting(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    assert!(
        context.block_production_time >= state.deadline_commitment_time,
        "Vote counting cannot start before specified starting time {} ms UTC, current time is {} ms UTC",
        state.deadline_commitment_time,
        context.block_production_time,
    );
    assert_eq!(
        zk_state.calculation_state,
        CalculationStatus::Waiting,
        "Vote counting must start from Waiting state, but was {:?}",
        zk_state.calculation_state,
    );

    (
        state,
        vec![],
        vec![ZkStateChange::start_computation(vec![SecretVarMetadata {
            variable_type: SecretVarType::CountedYesVotes,
        }])],
    )
}

/// Automatically called when the computation is completed
///
/// The only thing we do is to instantly open/declassify the output variables.
#[zk_on_compute_complete]
fn counting_complete(
    _context: ContractContext,
    state: ContractState,
    _zk_state: ZkState<SecretVarMetadata>,
    output_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    (
        state,
        vec![],
        vec![ZkStateChange::OpenVariables {
            variables: output_variables,
        }],
    )
}

/// Automatically called when a variable is opened/declassified.
///
/// We can now read the for and against variables, and compute the result
#[zk_on_variables_opened]
fn open_sum_variable(
    _context: ContractContext,
    mut state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    opened_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    assert_eq!(
        opened_variables.len(),
        1,
        "Unexpected number of output variables"
    );
    let votes_for = read_variable_u32_le(&zk_state, opened_variables.get(0));
    let total_votes = zk_state
        .secret_variables
        .iter()
        .filter(|x| x.metadata.variable_type == SecretVarType::Vote)
        .count();
    let votes_against = (total_votes as u32) - votes_for;

    let vote_result = determine_result(
        &state.vote_definition,
        state.allowed_voters.len() as u32,
        votes_for,
        votes_against,
    );
    state.vote_result = Some(vote_result.clone());

    if cfg!(feature = "attestation") {
        (
            state,
            vec![],
            vec![ZkStateChange::Attest {
                data_to_attest: serialize(vote_result),
            }],
        )
    } else {
        (state, vec![], vec![ZkStateChange::ContractDone])
    }
}

fn serialize<T: ReadWriteState>(it: T) -> Vec<u8> {
    let mut output: Vec<u8> = vec![];
    it.state_write_to(&mut output).expect("Could not serialize");
    output
}

/// Attestation complete
#[cfg(feature = "attestation")]
#[zk_on_attestation_complete]
fn handle_attestation(
    _context: ContractContext,
    state: ContractState,
    _zk_state: ZkState<SecretVarMetadata>,
    _attestation_id: AttestationId,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    (state, vec![], vec![ZkStateChange::ContractDone])
}

/// Reads a variable's data as an u32.
fn read_variable_u32_le(
    zk_state: &ZkState<SecretVarMetadata>,
    sum_variable_id: Option<&SecretVarId>,
) -> u32 {
    let sum_variable_id = *sum_variable_id.unwrap();
    let sum_variable = zk_state.get_variable(sum_variable_id).unwrap();
    let mut buffer = [0u8; 4];
    buffer.copy_from_slice(sum_variable.data.as_ref().unwrap().as_slice());
    <u32>::from_le_bytes(buffer)
}

fn determine_result(
    def: &VoteBasis,
    num_registered_voters: u32,
    votes_for: u32,
    votes_against: u32,
) -> VoteResult {
    let votes_total = if def.absent_as_against {
        num_registered_voters
    } else {
        votes_for + votes_against
    };
    let vote_ratio = Fraction::new(votes_for, votes_total);
    let passed = vote_ratio > def.required_ratio;

    VoteResult {
        votes_for,
        votes_against,
        passed,
    }
}
