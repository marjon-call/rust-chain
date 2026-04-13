# rust-chain

A proof-of-stake L1 blockchain being built from scratch in Rust. The goal of this project is to learn L1 security by designing and implementing a production-grade L1 end-to-end — consensus, networking, state, mempool, and RPC — so that every layer of the stack is understood deeply enough to reason about its attack surface.

**This project is still in active development** and is not feature-complete. Expect breaking changes, missing pieces, and rough edges.

It currently includes a p2p gossip network, a mempool, block production by staked validators, and a JSON-RPC server for interacting with the chain.

## Features

- **PoS consensus** — validators stake tokens via `Stake` transactions and are selected to propose blocks.
- **p2p networking** — libp2p with gossipsub + mDNS for transaction and block propagation.
- **Mempool** — pending transactions are gossiped between peers and included by the next block producer.
- **Wallets & signatures** — secp256k1 (k256) keys, addresses derived from the public key.
- **JSON-RPC server** — axum-based HTTP endpoint for querying chain state and submitting transactions.
- **Genesis config** — initial supply, initial stake, and chain ID loaded from `genesis.json`.

## Project layout

```
src/
├── main.rs           # entrypoint: loads genesis, starts RPC, block production, p2p node
├── chain/
│   ├── block.rs      # block structure
│   ├── blockchain.rs # chain state machine
│   ├── state.rs      # account + validator state
│   ├── mempool.rs    # pending tx pool
│   ├── genesis.rs    # genesis config loader
│   └── validator.rs  # validator selection
├── network/node.rs   # libp2p node + block production loop
├── rpc/server.rs     # JSON-RPC HTTP server
└── types/
    ├── transaction.rs
    └── wallet.rs
```

## Build & run

```bash
cargo run
```

Environment variables:

- `RPC_PORT` — port for the JSON-RPC server (default `8545`).

Run multiple nodes on one machine by giving each a different `RPC_PORT`; they will discover each other over mDNS.

On startup, `main.rs` prints ready-to-paste `curl` commands for a sample stake and transfer transaction signed by the test wallet baked into `genesis.json`.

## JSON-RPC methods

All requests are `POST /` with body `{"method": "...", "params": [...], "id": 1}`.

| Method | Params | Description |
| --- | --- | --- |
| `getBlockNumber` | — | Current chain height. |
| `getBalance` | `[address]` | Balance of an account. |
| `getBlockByNumber` | `[index]` | Full block at the given index. |
| `sendRawTransaction` | `[hex]` | Submit a bincode-serialized, hex-encoded signed transaction to the mempool. |
| `getValidators` | — | Active validator set with stakes. |

Example:

```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"method": "getBlockNumber", "params": [], "id": 1}'
```

## Genesis

`genesis.json` defines the initial funded account, its initial stake, supply, and chain ID:

```json
{
  "initial_address": "f0090076474224898b1ac856772e8e7077845f40",
  "initial_supply": 1000,
  "initial_stake": 200,
  "chain_id": 67
}
```

The matching private key for the initial address is committed in `main.rs` for local testing only — **do not reuse it for anything real**.

## Status

Experimental / educational. Not audited, not production-ready.
