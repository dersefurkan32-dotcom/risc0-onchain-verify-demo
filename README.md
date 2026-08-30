# risc0-onchain-verify-demo

**End-to-end zkVM pipeline, built and verified on a laptop: Rust guest → real Groth16 proof (CPU + docker shrink-wrap) → Solidity verifier.**

This is the working artifact behind my zkVM/coprocessor security work. It exists
because the bugs that pay in this lane live exactly on this seam: what the guest
proves, what the journal commits to, and what the on-chain verifier actually
checks.

## What's inside

### `identity-bind/` — the guest + host (risc0-zkvm 3.0.6)

A minimal coprocessor-shaped program: it only proves a withdrawal when the
identity is bound (`owner == expected`) **and** the collateral rule holds
(`amount <= pledged`). The rule lives *inside* the proof, not in a contract.

```bash
cd identity-bind
RISC0_DEV_MODE=1 cargo run --release                    # dev mode: instant
cargo run --release -- --groth16 --save receipt.json    # real proof (needs docker for shrink-wrap)
cargo run --release -- --mismatch                       # guest panics: no proof exists
cargo run --release -- --overdraw                       # guest panics: no proof exists
```

`receipt-groth16.json` is a real receipt produced by this code (1.6 KB seal).

### `onchain-verify/` — the Solidity side (risc0-ethereum v3.0.1)

Deploys `RiscZeroGroth16Verifier` and verifies that real receipt on-chain:

```bash
cd onchain-verify
forge test -vv
```

```
[PASS] test_groth16_receipt_verifies_onchain() (gas: 243225)   # real pairing, real seal
[PASS] test_tampered_journal_reverts()                          # amount 100→101: REVERTS
[PASS] test_wrong_image_id_reverts()                            # different guest: REVERTS
```

The negative tests are the point: the verifier binds the exact guest image and
the exact journal. Everything else reverts.

## Why this matters (the security view)

Every zkVM integration audit starts with the same five questions:

1. Does the contract pin the **image ID** (or can any program's proof pass)?
2. Is the **journal** bound to the public inputs the prover claims (context,
   chain id, block state)?
3. Are the **verifier parameters** in sync with the prover (params drift)?
4. Can a receipt be **replayed** in a different context?
5. Where can execution **diverge** between host observation and guest execution?

This repo is the harness I use to answer them on real code.

## Requirements

- [Foundry](https://getfoundry.sh) for `onchain-verify/` (fully local, no RPC/keys)
- [RISC Zero](https://dev.risczero.com) (`rzup`) + Docker for producing new proofs
  in `identity-bind/` (the committed receipt + tests run without either)
