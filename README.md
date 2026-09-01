# RISC Zero on-chain verify lab

[![ci](https://github.com/dersefurkan/risc0-onchain-verify-demo/actions/workflows/ci.yml/badge.svg)](https://github.com/dersefurkan/risc0-onchain-verify-demo/actions/workflows/ci.yml)

End-to-end zkVM seam, local only: **Rust guest → Groth16 receipt → Solidity verifier**.

The bugs that matter in this lane live on that seam: what the guest proves, what the journal commits to, and what the on-chain verifier actually checks.

## Layout

```
identity-bind/     guest + host (risc0-zkvm 3.0.6)
onchain-verify/    Foundry tests against RiscZeroGroth16Verifier (risc0-ethereum v3.0.1)
```

## Guest (`identity-bind/`)

A coprocessor-shaped program. It only proves a withdrawal when identity is bound (`owner == expected`) **and** the collateral rule holds (`amount <= pledged`). The rule lives inside the proof, not in a contract.

```bash
cd identity-bind
RISC0_DEV_MODE=1 cargo run --release                 # instant, no real proof
cargo run --release -- --help
cargo run --release -- --groth16 --save receipt.json # real proof (Docker for shrink-wrap)
cargo run --release -- --mismatch                    # guest panics: no receipt
cargo run --release -- --overdraw                    # guest panics: no receipt
```

`identity-bind/receipt-groth16.json` is a committed real receipt from this code (1.6 KB seal). Reproducing a new Groth16 receipt needs RISC Zero (`rzup`) and Docker. The on-chain tests do **not**.

## On-chain (`onchain-verify/`)

Deploys `RiscZeroGroth16Verifier` and checks the committed receipt:

```bash
cd onchain-verify
forge test -vv
```

```
[PASS] test_groth16_receipt_verifies_onchain()   # real pairing, real seal
[PASS] test_tampered_journal_reverts()           # amount 100→101: reverts
[PASS] test_wrong_image_id_reverts()             # different guest: reverts
```

The negative tests are the point: the verifier binds the exact guest image and the exact journal. Everything else reverts.

CI runs only this Foundry suite. It does not rebuild the guest or re-prove.

## Why this seam matters

Every zkVM integration review starts with the same five questions:

1. Does the contract pin the **image ID** (or can any program's proof pass)?
2. Is the **journal** bound to the public inputs the prover claims (context, chain id, block state)?
3. Are the **verifier parameters** in sync with the prover?
4. Can a receipt be **replayed** in a different context?
5. Where can execution **diverge** between host observation and guest execution?

This repo is the harness used to answer them on real code.

## Requirements

- [Foundry](https://getfoundry.sh) for `onchain-verify/` (fully local, no RPC/keys)
- [RISC Zero](https://dev.risczero.com) (`rzup`) + Docker only if you want to produce a **new** proof

## Scope

Authorized research and teaching. No mainnet transactions. See [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE). RISC Zero and OpenZeppelin code under `onchain-verify/lib/` keep their upstream licenses.
