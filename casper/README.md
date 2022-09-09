#### Structure

* erc20 contains token contract.
  * erc20/erc20 is a adapted casper-ecosystem/erc20 repository with extended functionality for Pause, Reclaim, Claim, Mint, Burn. It's under Apache licence so should be fine.
* sign_cli helper executable for generating signature for testing purpose only
* tests contains tests for contracts (only for erc20 claim for now)


#### Prerequisites

```bash
### Ubuntu example

# Installing rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
rustup target add wasm32-unknown-unknown

# Installing just utility simillar to make
cargo install just

# Installing binaryen utilities for wasm optimization
apt-get install binaryen
```

#### Running tests
`just test`

#### Building contract
```
just build
ls contracts
```