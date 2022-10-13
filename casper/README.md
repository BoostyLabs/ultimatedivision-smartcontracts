#### Structure

* erc20 contains token contract.
  * erc20/erc20 is a adapted casper-ecosystem/erc20 repository with extended functionality for Pause, Reclaim, Claim, Mint, Burn. It's under Apache licence so should be fine.
* verifier - library that helps for verification signed message by go back-end. The library has tests and cli utility for manual testing.


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

#### Verificationt tests:
```
cd verifier
cargo test --release
```

#### Manual signature testing
```
cd verifier
cargo run --features="cli"


Example for nft:
cargo run --features="cli" nft -k 02f083a879b53f9c013425dfeaa804731300dc37473940908c2150bc6a2243a548 -u account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e -c hash-1d2f5eed581e3750fa3d2fd15ef782aa66a55a679346c0a339c485c78fc9fe68 -t 666 -s 4c7816baca358097052eaa8d313fd2e621c385ed4653750a550dcd5865a20cd231dc880727f8429dc1a30deb93968bbf5a90b1a452552a145a748a00ed0b71031b

Example for token signature:
cargo run --features="cli" token -k 02f083a879b53f9c013425dfeaa804731300dc37473940908c2150bc6a2243a548 -u account-hash-9060c0820b5156b1620c8e3344d17f9fad5108f5dc2672f2308439e84363c88e -c hash-5aed0843516b06e4cbf56b1085c4af37035f2c9c1f18d7b0ffd7bbe96f91a3e0 -v 5000 -n 0 -s a3f92029dae8b7a1fd682784995bd2fd3a395fe408c4eef6cccc358e7981b728625a6bb0a3bb2d91c4355ee7054bf9a2eef3aa8b31d63275eee02202d77a146a1b


where:
-k - verification key in hex format
-u - user hash in format account-hash-$hash
-c - contract hash in format hash-$hash
-v - value as U256
-n - nonce as u64
-t - token id as u64
-s - signature in hex format
```

#### Building contract
```
just build
ls contracts
```