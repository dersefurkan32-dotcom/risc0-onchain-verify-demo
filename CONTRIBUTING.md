# Contributing

Keep the seam intact: one guest, one committed Groth16 receipt, three on-chain tests (happy path + tampered journal + wrong image ID).

## On-chain tests (required)

```bash
cd onchain-verify
forge fmt
forge test -vv
```

CI runs this path only.

## Guest / host (optional)

Rebuilding the guest or producing a new Groth16 receipt needs [RISC Zero](https://dev.risczero.com) and Docker. If you change the guest, you **must** regenerate `identity-bind/receipt-groth16.json` and update `IMAGE_ID` / `SEAL` / `JOURNAL` in `onchain-verify/test/IdentityBindOnchainVerify.t.sol` in the same change. Otherwise the on-chain tests will fail.

```bash
cd identity-bind
RISC0_DEV_MODE=1 cargo run --release -- --help
```

## Rules

- Do not vendor extra copies of `lib/`.
- Do not add live RPC or private keys.
- Do not weaken the two negative tests.
- Commit messages: short imperative (`Pin Foundry in CI`).
