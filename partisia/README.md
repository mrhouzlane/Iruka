# Partisia Rust example contracts

You can always find the newest documentation on the [main site](https://partisiablockchain.gitlab.io/documentation).


## Prerequisites

To develop and compile contracts for the Partisia Blockchain, you need to install Rust. 
To install Rust for you platform follow the instructions on <https://rustup.rs/>.

To compile contracts you will also need to install [Git](https://git-scm.com/downloads).

If you need to develop zero-knowledge contracts then you will also need to install [Java 17](https://openjdk.org/) to run the zk-compiler.

If Working from a Windows machine you must [get Visual Studio with C++  build tools](https://visualstudio.microsoft.com/downloads/)
- In Visual Studio Installer choose *Desktop development with C++*.

## Compile and install the cargo `partisia-contract` command

The partisia-contract tool is a small application that helps you compile a contract.
To compile it and install it using cargo run:

```bash
cargo install cargo-partisia-contract
```

Test that it worked by executing: `cargo partisia-contract --version`. This should print the version of the tool.

## Compiling a contract

To compile a contract you simply change directory to one of the rust-example-contracts and compile: 
```bash
cd ..
cd contracts/token/
cargo partisia-contract build --release
```

The `--release` flag compiles the contract in release mode and can be excluded to only test the compilation of the contract.

### Zero-Knowledge contracts

The zk-contracts consist of two main parts. The contract itself as well as a zero-knowledge computation. 
The partisia-contract tool is able to detect and compile both these parts together. To do so the tool fetches the zk-compiler.jar and runs it.

```bash
cd ..
cd contracts/example-zk-voting/
cargo partisia-contract build --release
```

This creates the linked byte file with the zk wasm file (.zkwa) as well as the wasm file.

## Included example contracts

There are contracts included in the zip as well as zero knowledge contracts.

The included normal contracts are:

1. An ERC20 token contract located in `contracts/token`
2. A general purpose voting contract located in `contracts/voting`
3. An auction contract that sells ERC20 tokens of one type for another located in `contracts/auction`
4. An NFT contract located in `contracts/nft`
5. An escrow contract that transfers tokens when a condition is met, located in `contract/conditional-escrow-transfer`
6. A liquidity swap contract that exchanges one type of tokens for another, located in `contract/liquidity-swap`
7. A contract that deploys voting contracts located in `contracts/multi-voting`

The included zk-contracts are:

1. A secret voting contract located in `contracts/zk-voting`
2. An average salary contract located in `contracts/zk-average-salary`
3. A second price auction contract located in `contracts/zk-second-price-auction`

Multiple of the examples are described in great detail on the [main site](https://partisiablockchain.gitlab.io/documentation).

## How to write your own contract

For writing your own contract refer to the documentation on the [main site](https://partisiablockchain.gitlab.io/documentation).