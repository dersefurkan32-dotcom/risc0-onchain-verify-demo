# Security policy

This repository is a **local zkVM lab**. It verifies a committed Groth16 receipt on a local Foundry EVM. It does not talk to a live chain.

## Authorized use only

- Run `forge test` in `onchain-verify/` and `cargo run` in `identity-bind/` on your machine.
- Do not broadcast proofs, receipts, or verifier deployments to public networks from this lab unless you own the target and intend to.
- Do not use this as a generic “any RISC Zero app is broken” kit. Findings against other systems need written authorization.

## Reporting a problem in *this* lab

If the committed receipt stops verifying, an image ID drifts, or a negative test no longer reverts, email **dersefurkan32@gmail.com** with:

- `forge` / `cargo` versions
- the exact command
- whether you regenerated `receipt-groth16.json`

Do not file a public issue for a vulnerability in a third-party zkVM integration.

## Secrets

Never commit RPC keys, proving-service tokens, or `.env` files. The committed receipt is a public proof object for this demo guest, not a credential.
