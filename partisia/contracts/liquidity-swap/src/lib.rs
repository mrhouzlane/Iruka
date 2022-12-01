//! This is an example simple liquidity swap smart contract.
//! It is simplified implication of UniSwap v2:
//! https://docs.uniswap.org/protocol/V2/concepts/protocol-overview/how-uniswap-works
//!
//! The contracts exchanges (or swaps) between two types of tokens,
//! with an the exchange rate as given by the `constant product formula: x * y = k`.
//! We consider `x` to be the balance of token pool A and `y` to be the balance of token pool B.
//! `k` represents the invariant (`swap_constant`) that must be upheld between swaps.
//!
//! In order to perform a swap between the two desired tokens, the owner must first initialize
//! both token pools, `initialize_pool_{a,b}`, by transferring an amount of tokens to both pools via a transfer call to
//! the corresponding token contract. This will also initialize the (final) value of `k`.
//!
//! User's (including the owner) can then `deposit` tokens to the contract, which can be used to
//! exchange to the opposite token. This is done by calling `swap`. `swap` will calculate the
//! amount of tokens to convert of the incoming token to the opposite token, based on the above formula.
//! A user may then `withdraw` the resulting tokens of the swap (or simply his own deposited tokens).
//!
//! Finally, the owner of the contract may close the pools, `close_pools`, by transferring both token pools to his own account,
//! effectively closing the contract. Only valid withdrawals are allowed in the closed state.
//!
//! Both `deposit` and `withdraw` makes use of `transfer` calls to the token contract, which
//! are ensured to be successful via callbacks.
//!
//! Because the relative price of the two tokens can only be changed through swapping,
//! divergences between the prices of the current contract and the prices of similar external contracts create arbitrage opportunities.
//! This mechanism ensures that the contract's prices always trend toward the market-clearing price.
//!
//! The two token contracts linked to this contract must currently be owned by the same owner,
//! as this contract.
#![allow(unused_variables)]

mod tests;

#[macro_use]
extern crate pbc_contract_codegen;
extern crate core;

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::address::{Address, AddressType, Shortname};
use pbc_contract_common::context::{CallbackContext, ContractContext};
use pbc_contract_common::events::EventGroup;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;
use std::collections::BTreeMap;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, ReadWriteRPC)]
/// Enum for token types
pub enum Token {
    /// The value representing token A.
    A = 0,
    /// The value representing token B.
    B = 1,
}

#[cfg(feature = "abi")]
impl pbc_traits::CreateTypeSpec for Token {
    fn __ty_name() -> String {
        u8::__ty_name()
    }

    fn __ty_identifier() -> String {
        u8::__ty_identifier()
    }

    fn __ty_spec_write(w: &mut Vec<u8>, lut: &BTreeMap<String, u8>) {
        u8::__ty_spec_write(w, lut)
    }
}

/// Constants for deducing token types
const TOKEN_A: Token = Token::A;
const TOKEN_B: Token = Token::B;

/// A token pool that holds tokens which can be swapped by users.
///
/// ### Fields:
///
/// * `token_address`: [`Address`] - The address of the token contract.
///
/// * `pool`: [`u64`] - The amount of tokens a token pool has.
#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq)]
pub struct TokenPool {
    token_address: Address,
    pool: u64,
}

/// Holds user balances for the two tokens.
/// Keeps track of how much of a given token a user can withdraw.
///
/// ### Fields:
///
/// * `pool_a_balance`: [`u64`] - the amount of token A that a user can withdraw from the contract.
///
/// * `pool_b_balance`: [`u64`] - the amount of token B that a user can withdraw from the contract.
#[derive(ReadWriteState, CreateTypeSpec, Clone, PartialEq, Eq)]
pub struct UserBalance {
    pool_a_balance: u64,
    pool_b_balance: u64,
}

impl UserBalance {
    fn get_mut_balance_for(&mut self, token: Token) -> &mut u64 {
        if token == TOKEN_A {
            &mut self.pool_a_balance
        } else {
            &mut self.pool_b_balance
        }
    }
}

/// This is the state of the contract which is persisted on the chain.
///
/// The #\[state\] macro generates serialization logic for the struct.
///
/// ### Fields:
///
/// * `contract_owner`: [`Address`] - The owner of the contract.
///
/// * `token_pool_a`: [`Address`] - The address of the first token contract.
///
/// * `token_pool_b`: [`Address`] - The address of the second token contract.
///
/// * `swap_constant`: [`u64`] - The invariant used to calculate exchange rates.
///    It's based on the 'constant product formula': x * y = k, k being the swap_constant.
///
/// * `user_balances`: [`BTreeMap<Address, UserBalance>`] - The map containing all token balances of all users of the contract.
///
/// * `is_closed`: [`bool`] - Boolean indicating whether the contract is operable or not.
#[state]
pub struct LiquiditySwapContractState {
    contract_owner: Address,
    token_pool_a: TokenPool,
    token_pool_b: TokenPool,
    swap_constant: u64,
    user_balances: BTreeMap<Address, UserBalance>,
    is_closed: bool,
}

impl LiquiditySwapContractState {
    /// Adds tokens to the `user_balances` map of the contract.
    /// If the user isn't already present, creates an entry with an empty UserBalance.
    ///
    /// ### Parameters:
    ///
    /// * `user`: [`Address`] - The key of the entry.
    ///
    /// * `token`: [`Token`] - The token to add to.
    ///
    /// * `amount`: [`u64`] - The amount to add.
    ///
    fn add_to_user_balance(&mut self, user: Address, token: Token, amount: u64) {
        let user_balance = self.user_balances.entry(user).or_insert(UserBalance {
            pool_a_balance: 0,
            pool_b_balance: 0,
        });

        *user_balance.get_mut_balance_for(token) += amount;
    }

    /// Subtracts tokens from the `user_balances` map of the contract.
    /// Requires that the user already has an entry and that the subtraction yields a non-negative value.
    ///
    /// ### Parameters:
    ///
    /// * `user`: [`Address`] - The key of the entry.
    ///
    /// * `token`: [`Token`] - The token to subtract from.
    ///
    /// * `amount`: [`u64`] - The amount to subtract.
    ///
    fn subtract_from_user_balance(&mut self, user: Address, token: Token, amount: u64) {
        let user_balance = self
            .user_balances
            .get_mut(&user)
            .expect("Need existing balance");

        let token_balance = user_balance.get_mut_balance_for(token);
        let new_token_balance = token_balance
            .checked_sub(amount)
            .expect("Insufficient funds");

        *token_balance = new_token_balance;
    }

    /// Retrieves a copy of the pool that matches `token`.
    ///
    /// ### Parameters:
    ///
    /// * `token`: [`Token`] - The token matching the desired pool.
    ///
    /// # Returns
    /// A pool value of type [`u64`]
    fn get_pool_for(&self, token: Token) -> u64 {
        if token == TOKEN_A {
            self.token_pool_a.pool
        } else {
            self.token_pool_b.pool
        }
    }

    /// Retrieves a mutable reference to the pool that matches `token`.
    ///
    /// ### Parameters:
    ///
    /// * `token`: [`Token`] - The token matching the desired pool.
    ///
    /// # Returns
    /// A mutable pool value of type [`&mut u64`]
    fn get_mut_pool_for(&mut self, token: Token) -> &mut u64 {
        if token == TOKEN_A {
            &mut self.token_pool_a.pool
        } else {
            &mut self.token_pool_b.pool
        }
    }

    /// Retrieves a pair of tokens with the `input_token_address` being the "from"-token
    /// and the remaining token being "to".
    /// Requires that `input_token_address` matches the contract's pools.
    ///
    /// ### Parameters:
    ///
    /// * `token`: [`Token`] - The token matching the desired pool.
    ///
    /// # Returns
    /// The from/to-pair of tokens of type [`(Token, Token)`]
    fn deduce_from_to_tokens(&self, input_token_address: Address) -> (Token, Token) {
        let is_from_a = self.token_pool_a.token_address == input_token_address;
        let is_from_b = self.token_pool_b.token_address == input_token_address;
        if !is_from_a && !is_from_b {
            panic!("Provided invalid token address")
        }

        if is_from_a {
            (TOKEN_A, TOKEN_B)
        } else {
            (TOKEN_B, TOKEN_A)
        }
    }
}

/// Initialize the contract.
///
/// # Parameters
///
///   * `context`: [`ContractContext`] - The contract context containing sender and chain information.
///
///   * `token_a_address`: [`Address`] - The address of token A.
///
///   * `token_b_address`: [`Address`] - The address of token B.
///
///
/// The new state object of type [`LiquiditySwapContractState`] with all address fields initialized to their final state and remaining fields initialized to a default value.
///
#[init]
pub fn initialize(
    context: ContractContext,
    token_a_address: Address,
    token_b_address: Address,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert_eq!(
        token_a_address.address_type,
        AddressType::PublicContract,
        "Tried to provide a non-Public Contract token for token A"
    );
    assert_eq!(
        token_b_address.address_type,
        AddressType::PublicContract,
        "Tried to provide a non-Public Contract token for token B"
    );
    assert_ne!(
        token_a_address, token_b_address,
        "Cannot initialize swap with duplicate tokens"
    );

    let new_state = LiquiditySwapContractState {
        contract_owner: context.sender,
        token_pool_a: TokenPool {
            token_address: token_a_address,
            pool: 0,
        },
        token_pool_b: TokenPool {
            token_address: token_b_address,
            pool: 0,
        },
        swap_constant: 0,
        user_balances: BTreeMap::new(),
        is_closed: true,
    };

    (new_state, vec![])
}

/// Initialize pool {a, b} of the contract.
/// This can only be done by the contract owner and the contract has to be in its closed state.
///
/// ### Parameters:
///
///  * `context`: [`ContractContext`] - The contract context containing sender and chain information.
///
///  * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
///  * `token_address`: [`Address`] - The address of the token {a, b}.
///
///  * `pool_size`: [`u64`] - The desired size of token pool {a, b}.
///
/// # Returns
/// The unchanged state object of type [`LiquiditySwapContractState`].
#[action(shortname = 0x01)]
pub fn provide_liquidity(
    context: ContractContext,
    state: LiquiditySwapContractState,
    token_address: Address,
    pool_size: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert_eq!(
        context.sender, state.contract_owner,
        "Only the contract owner can initialize its pools"
    );
    assert!(
        state.is_closed,
        "Can only initialize when the contract is closed"
    );

    let (from_token, _) = state.deduce_from_to_tokens(token_address);
    let mut event_group_builder = EventGroup::builder();
    event_group_builder
        .call(token_address, token_contract_transfer_from())
        .argument(context.sender)
        .argument(context.contract_address)
        .argument(pool_size)
        .done();

    event_group_builder
        .with_callback(SHORTNAME_PROVIDE_LIQUIDITY_CALLBACK)
        .argument(from_token)
        .argument(pool_size)
        .done();

    (state, vec![event_group_builder.build()])
}

/// Handles callback from `provide_liquidity_{a,b}`.
/// If the transfer event is successful the corresponding pool is initialized.
/// If both pools have currency, the contract is declared open.
/// If the transfer event fails the state is unchanged.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`] - The contractContext for the callback.
///
/// * `callback_context`: [`CallbackContext`] - The callbackContext.
///
/// * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
/// * `token`: [`Token`] - Indicating the token pool to initialize
///
/// * `pool_size`: [`u64`] - The desired size of token pool {A, B}.
///
///
/// ### Returns
///
/// The updated state object of type [`LiquiditySwapContractState`], with the corresponding pool initialized and the contract opened if meeting the requirements.
#[callback(shortname = 0x10)]
pub fn provide_liquidity_callback(
    context: ContractContext,
    callback_context: CallbackContext,
    mut state: LiquiditySwapContractState,
    token: Token,
    pool_size: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert!(callback_context.success, "Transfer did not succeed");

    *state.get_mut_pool_for(token) += pool_size;

    // Check if both pools has been initialized. If so, open the contract and set the contract constant.
    if state.token_pool_a.pool > 0u64 && state.token_pool_b.pool > 0u64 {
        state.swap_constant = state.token_pool_a.pool * state.token_pool_b.pool;
        state.is_closed = false;
    }

    (state, vec![])
}

/// Deposit token A or B into the calling users balance on the contract.
/// If the contract is closed, the action fails.
///
/// ### Parameters:
///
///  * `context`: [`ContractContext`] - The contract context containing sender and chain information.
///
///  * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
///  * `token_address`: [`Address`] - The address of the deposited token contract.
///
///  * `amount`: [`u64`] - The amount to deposit.
///
/// # Returns
/// The unchanged state object of type [`LiquiditySwapContractState`].
#[action(shortname = 0x02)]
pub fn deposit(
    context: ContractContext,
    state: LiquiditySwapContractState,
    token_address: Address,
    amount: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert!(
        !state.is_closed,
        "Cannot make a deposit when the contract is closed"
    );

    let (from_token, _) = state.deduce_from_to_tokens(token_address);
    let mut event_group_builder = EventGroup::builder();
    event_group_builder
        .call(token_address, token_contract_transfer_from())
        .argument(context.sender)
        .argument(context.contract_address)
        .argument(amount)
        .done();

    event_group_builder
        .with_callback(SHORTNAME_DEPOSIT_CALLBACK)
        .argument(from_token)
        .argument(amount)
        .done();

    (state, vec![event_group_builder.build()])
}

/// Handles callback from `deposit`.
/// If the transfer event is successful the caller of `deposit` is added to the `state.user_balances`
/// adding `amount` to the `token` pool balance.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`] - The contractContext for the callback.
///
/// * `callback_context`: [`CallbackContext`] - The callbackContext.
///
/// * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
/// * `token`: [`Token`] - Indicating the token pool balance of which to add `amount` to.
/// * `amount`: [`u64`] - The desired amount to add to `token_type` pool balance.
///
///
/// ### Returns
///
/// The updated state object of type [`LiquiditySwapContractState`] with an updated entry for the caller of `deposit`.
#[callback(shortname = 0x20)]
pub fn deposit_callback(
    context: ContractContext,
    callback_context: CallbackContext,
    mut state: LiquiditySwapContractState,
    token: Token,
    amount: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert!(callback_context.success, "Transfer did not succeed");

    state.add_to_user_balance(context.sender, token, amount);

    (state, vec![])
}

/// Swap `amount` of token A or B to the opposite token at the exchange rate dictated by `the constant product formula`.
/// The swap is executed on the user balances of tokens for the calling user.
/// If the contract is closed or if the caller does not have a sufficient balance of the token, the action fails.
///
/// ### Parameters:
///
///  * `context`: [`ContractContext`] - The contract context containing sender and chain information.
///
///  * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
///  * `input_token_address`: [`Address`] - The address of the token contract being swapped from.
///
///  * `amount`: [`u64`] - The amount to swap of the token matching `input_token`.
///
/// # Returns
/// The updated state object of type [`LiquiditySwapContractState`] yielding the result of the swap.
#[action(shortname = 0x03)]
pub fn swap(
    context: ContractContext,
    mut state: LiquiditySwapContractState,
    input_token_address: Address,
    amount: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert!(
        !state.is_closed,
        "Cannot make a swap when the contract is closed"
    );
    let (token_from, token_to) = state.deduce_from_to_tokens(input_token_address);
    let from_pool_value = state.get_pool_for(token_from);
    let to_pool_value = state.get_pool_for(token_to);

    state.subtract_from_user_balance(context.sender, token_from, amount);
    let new_from_pool_value = from_pool_value + amount;
    let new_to_pool_value = u64_division_ceil(state.swap_constant, new_from_pool_value);

    state.add_to_user_balance(context.sender, token_to, to_pool_value - new_to_pool_value);
    *state.get_mut_pool_for(token_from) = new_from_pool_value; // Update from pool
    *state.get_mut_pool_for(token_to) = new_to_pool_value; // Update to pool

    (state, vec![])
}

/// Withdraw `amount` of token A or B from the contract for the calling user.
/// This fails if `amount` is larger than the user balance of the corresponding token.
///
/// It preemptively updates the state of the user's balance before making the transfer.
/// This means that if the transfer fails, the contract could end up with more money than it has registered, which is acceptable.
/// This is to incentivize the user to spend enough gas to complete the transfer.
///
/// ### Parameters:
///
///  * `context`: [`ContractContext`] - The contract context containing sender and chain information.
///
///  * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
///  * `token_address`: [`Address`] - The address of the token contract to withdraw to.
///
///  * `amount`: [`u64`] - The amount to withdraw.
///
/// # Returns
/// The unchanged state object of type [`LiquiditySwapContractState`].
#[action(shortname = 0x04)]
pub fn withdraw(
    context: ContractContext,
    mut state: LiquiditySwapContractState,
    token_address: Address,
    amount: u64,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    let (token_from, _) = state.deduce_from_to_tokens(token_address);

    state.subtract_from_user_balance(context.sender, token_from, amount);

    let mut event_group_builder = EventGroup::builder();
    event_group_builder
        .call(token_address, token_contract_transfer())
        .argument(context.sender)
        .argument(amount)
        .done();

    (state, vec![event_group_builder.build()])
}

/// Empties the pools into the contract owner's balance and closes the contract.
/// Fails if called by anyone but the contract owner.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`] - The context for the action call.
///
/// * `state`: [`LiquiditySwapContractState`] - The current state of the contract.
///
/// ### Returns
///
/// The updated state object of type [`LiquiditySwapContractState`].
#[action(shortname = 0x05)]
pub fn close_pools(
    context: ContractContext,
    mut state: LiquiditySwapContractState,
) -> (LiquiditySwapContractState, Vec<EventGroup>) {
    assert_eq!(
        context.sender, state.contract_owner,
        "Only the contract owner can close the pools"
    );
    assert!(!state.is_closed, "The contract is already closed");

    state.add_to_user_balance(state.contract_owner, TOKEN_A, state.token_pool_a.pool);
    state.add_to_user_balance(state.contract_owner, TOKEN_B, state.token_pool_b.pool);

    // Close contract
    state.token_pool_a.pool = 0;
    state.token_pool_b.pool = 0;
    state.is_closed = true;

    (state, vec![])
}

/// * HELPER FUNCTIONS *

/// Creates the `Shortname` corresponding to the `transfer` action of a token contract.
/// This is utilized in combination with an `EventGroupBuilder`'s `call` function.
///
/// ### Returns:
///
/// The `Shortname` corresponding to the `transfer` action of a token contract.
#[inline]
fn token_contract_transfer() -> Shortname {
    Shortname::from_u32(0x01)
}

/// Creates the `Shortname` corresponding to the `transfer_from` action of a token contract.
/// This is utilized in combination with an `EventGroupBuilder`'s `call` function.
///
/// ### Returns:
///
/// The `Shortname` corresponding to the `transfer_from` action of a token contract.
#[inline]
fn token_contract_transfer_from() -> Shortname {
    Shortname::from_u32(0x03)
}

/// Divides two [`u64`] types and rounds up.
///
/// ### Parameters:
///
/// * `numerator`: [`u64`] - The numerator for the division.
///
/// * `denominator`: [`u64`] - The denominator for the division.
///
/// ### Returns:
///
/// The result of the division, rounded up, of type [`u64`].
fn u64_division_ceil(numerator: u64, denominator: u64) -> u64 {
    numerator / denominator + u64::from(numerator % denominator > 0)
}
