# Substrate RedPacket Module
  
  RedPacket(红包) is a easy way for airdropping. You can create a RedPacket that reserve some funds, others can claim partial funds from the RedPacket for free. Finally, the RedPacket can be distributed by creator after expiration or after claiming all funds.


## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## Run

### Single Node Development Chain

Purge any exting developer chain state:

```bash
./target/release/redpacket purge-chain --dev
```

Start a development chain with:

```bash
./target/release/redpacket --dev
```
